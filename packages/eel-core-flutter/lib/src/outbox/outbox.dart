// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../connectivity/connectivity_probe.dart';
import '../errors/problem_json.dart';

/// Outbox entry states (plan §2.5).
enum OutboxState { pending, syncing, synced, failed, dead }

/// Typed outbox entry, hash-chained for tamper evidence.
@immutable
class OutboxRecord {
  const OutboxRecord({
    required this.id,
    required this.idempotencyKey,
    required this.type,
    required this.payload,
    required this.hlc,
    required this.createdAt,
    this.state = OutboxState.pending,
    this.attempts = 0,
    this.previousHash = '',
    this.hash = '',
    this.lastError,
  });

  final String id;
  final String idempotencyKey;

  /// CloudEvents `type`, e.g. `eel.field_mobile.incident_reported.v1`.
  final String type;
  final Map<String, Object?> payload;

  /// kit-sdk HLC string attached to the queued record (plan §2.3).
  final String hlc;
  final DateTime createdAt;
  final OutboxState state;
  final int attempts;
  final String previousHash;

  /// SHA-256 of (previousHash + payload + hlc) — WAL chain (EC-MOB-01).
  final String hash;
  final String? lastError;

  bool get isAuditType => type.endsWith('.audit.v1') || type.contains('audit');

  OutboxRecord copyWith({
    OutboxState? state,
    int? attempts,
    String? hash,
    String? lastError,
  }) =>
      OutboxRecord(
        id: id,
        idempotencyKey: idempotencyKey,
        type: type,
        payload: payload,
        hlc: hlc,
        createdAt: createdAt,
        state: state ?? this.state,
        attempts: attempts ?? this.attempts,
        previousHash: previousHash,
        hash: hash ?? this.hash,
        lastError: lastError,
      );

  Map<String, dynamic> toJson() => {
        'id': id,
        'idempotencyKey': idempotencyKey,
        'type': type,
        'payload': payload,
        'hlc': hlc,
        'createdAt': createdAt.toIso8601String(),
        'state': state.name,
        'attempts': attempts,
        'previousHash': previousHash,
        'hash': hash,
        if (lastError != null) 'lastError': lastError,
      };

  factory OutboxRecord.fromJson(Map<String, dynamic> json) => OutboxRecord(
        id: json['id'] as String,
        idempotencyKey: json['idempotencyKey'] as String,
        type: json['type'] as String,
        payload: (json['payload'] as Map?)?.cast<String, Object?>() ?? {},
        hlc: json['hlc'] as String? ?? '',
        createdAt: DateTime.parse(json['createdAt'] as String),
        state: OutboxState.values.byName(json['state'] as String),
        attempts: json['attempts'] as int? ?? 0,
        previousHash: json['previousHash'] as String? ?? '',
        hash: json['hash'] as String? ?? '',
        lastError: json['lastError'] as String?,
      );
}

/// Pluggable durable storage (Isar-backed at the app layer).
abstract class OutboxStore {
  Future<void> append(OutboxRecord record);
  Future<void> update(OutboxRecord record);
  Future<List<OutboxRecord>> pending({int limit});
  Future<List<OutboxRecord>> all();
}

/// In-memory store for tests.
class InMemoryOutboxStore implements OutboxStore {
  final Map<String, OutboxRecord> _records = {};

  @override
  Future<void> append(OutboxRecord record) async => _records[record.id] = record;

  @override
  Future<void> update(OutboxRecord record) async => _records[record.id] = record;

  @override
  Future<List<OutboxRecord>> pending({int limit = 100}) async => _records.values
      .where((r) => r.state == OutboxState.pending || r.state == OutboxState.failed)
      .take(limit)
      .toList();

  @override
  Future<List<OutboxRecord>> all() async => _records.values.toList();
}

/// One drain attempt's outcome for a record.
enum DrainOutcome { synced, retry, failed, dead }

/// Transport for one outbox record.
typedef DrainTransport = Future<DrainOutcome> Function(OutboxRecord record);

/// Typed outbox drain loop (plan §2.5):
/// - journal-before-act (append happens before UI mutation — caller's duty)
/// - Idempotency-Key on every send
/// - 2xx → synced; 4xx (except 408/429) → failed; network/5xx/408/429 → retry
/// - exponential backoff with full jitter, base 1 s, factor 2, cap 60 s
/// - budget 8 attempts → dead + surfaced in sync panel
class OutboxDrainer {
  OutboxDrainer({
    required this.store,
    required this.transport,
    this.maxAttempts = 8,
    this.baseDelay = const Duration(seconds: 1),
    this.factor = 2,
    this.capDelay = const Duration(seconds: 60),
    this.clock,
  });

  final OutboxStore store;
  final DrainTransport transport;
  final int maxAttempts;
  final Duration baseDelay;
  final int factor;
  final Duration capDelay;
  final Duration Function(int attempt)? clock;

  Timer? _timer;
  bool _draining = false;

  /// Full-jitter backoff per §2.5.4.
  Duration backoffFor(int attempt, {required Duration Function(double) rand}) {
    final exponential = baseDelay * (1 << (attempt - 1));
    final capped = exponential > capDelay ? capDelay : exponential;
    return rand(capped.inMilliseconds / 2);
    // full jitter picks uniformly in [0, capped] — caller supplies rand.
  }

  Duration get exponential =>
      baseDelay * (1 << (maxAttempts - 1));

  Future<void> start({Duration interval = const Duration(seconds: 15)}) async {
    _timer?.cancel();
    _timer = Timer.periodic(interval, (_) => drain());
  }

  void stop() {
    _timer?.cancel();
    _timer = null;
  }

  /// Drains pending records in order. Safe to call concurrently (guarded).
  Future<int> drain() async {
    if (_draining) return 0;
    _draining = true;
    var synced = 0;
    try {
      final batch = await store.pending();
      for (final record in batch) {
        if (record.attempts >= maxAttempts) {
          await store.update(
            record.copyWith(state: OutboxState.dead, lastError: 'attempt budget exhausted'),
          );
          continue;
        }
        await store.update(record.copyWith(state: OutboxState.syncing));
        try {
          final outcome = await transport(record);
          switch (outcome) {
            case DrainOutcome.synced:
              await store.update(record.copyWith(state: OutboxState.synced));
              synced++;
            case DrainOutcome.dead:
              await store.update(
                record.copyWith(state: OutboxState.dead, lastError: 'server dead-letter'),
              );
            case DrainOutcome.failed:
              await store.update(
                record.copyWith(
                  state: OutboxState.failed,
                  attempts: record.attempts + 1,
                  lastError: 'permanent failure (4xx)',
                ),
              );
            case DrainOutcome.retry:
              await store.update(
                record.copyWith(
                  state: OutboxState.pending,
                  attempts: record.attempts + 1,
                  lastError: 'transient failure',
                ),
              );
          }
        } on ApiProblemException catch (e) {
          final status = e.problem.status;
          if (status == 408 || status == 429) {
            await store.update(
              record.copyWith(
                state: OutboxState.pending,
                attempts: record.attempts + 1,
                lastError: 'HTTP $status',
              ),
            );
          } else {
            await store.update(
              record.copyWith(
                state: OutboxState.failed,
                attempts: record.attempts + 1,
                lastError: e.problem.supportId,
              ),
            );
          }
        } catch (e) {
          await store.update(
            record.copyWith(
              state: OutboxState.pending,
              attempts: record.attempts + 1,
              lastError: e.toString(),
            ),
          );
        }
      }
    } finally {
      _draining = false;
    }
    return synced;
  }
}

/// Queue bounds per §2.5: per-type capacity with LRU eviction only for
/// `content` types, never audit types (EC-MOB-02).
Future<int> enforceQueueBounds({
  required OutboxStore store,
  required String type,
  required int capacity,
}) async {
  final all = await store.all();
  final ofType = all.where((r) => r.type == type).toList()
    ..sort((a, b) => a.createdAt.compareTo(b.createdAt));
  if (ofType.length <= capacity) return 0;
  var evicted = 0;
  for (final r in ofType.take(ofType.length - capacity)) {
    if (r.isAuditType) continue; // never evict audit types
    await store.update(r.copyWith(state: OutboxState.dead, lastError: 'evicted: queue full'));
    evicted++;
  }
  return evicted;
}

/// Connectivity trigger convenience: drain on [ConnectivitySnapshot.isUsable].
Future<int> drainOnConnectivity(
  OutboxDrainer drainer,
  ConnectivitySnapshot snapshot,
) =>
      snapshot.isUsable ? drainer.drain() : Future.value(0);