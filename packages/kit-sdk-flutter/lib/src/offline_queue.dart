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

enum OutboxEntryState { pending, syncing, synced, failed, conflict }

class OutboxEntry {
  OutboxEntry({
    required this.id,
    required this.idempotencyKey,
    required this.payload,
    required this.createdAt,
    this.state = OutboxEntryState.pending,
    this.stepCommitted = false,
  });

  final String id;
  final String idempotencyKey;
  final Map<String, Object?> payload;
  final DateTime createdAt;
  OutboxEntryState state;

  /// Write-ahead log flag: true after both journal write and commit write.
  bool stepCommitted;

  bool get isExpired =>
      DateTime.now().difference(createdAt) > const Duration(days: 7);

  Map<String, Object?> toJson() => {
        'id': id,
        'idempotencyKey': idempotencyKey,
        'payload': payload,
        'createdAt': createdAt.toIso8601String(),
        'state': state.name,
        'stepCommitted': stepCommitted,
      };

  factory OutboxEntry.fromJson(Map<String, Object?> j) => OutboxEntry(
        id: j['id'] as String,
        idempotencyKey: j['idempotencyKey'] as String,
        payload: (j['payload'] as Map?)?.cast<String, Object?>() ?? {},
        createdAt: DateTime.parse(j['createdAt'] as String),
        state: OutboxEntryState.values.byName(j['state'] as String),
        stepCommitted: j['stepCommitted'] as bool? ?? false,
      );
}

/// Pluggable persistence interface for the offline queue.
abstract class OutboxStorage {
  Future<void> save(OutboxEntry entry);
  Future<void> remove(String id);
  Future<List<OutboxEntry>> loadAll();
}

/// In-memory storage for tests and stubs.
class InMemoryOutboxStorage implements OutboxStorage {
  final Map<String, OutboxEntry> _store = {};

  @override
  Future<void> save(OutboxEntry entry) async => _store[entry.id] = entry;

  @override
  Future<void> remove(String id) async => _store.remove(id);

  @override
  Future<List<OutboxEntry>> loadAll() async => _store.values.toList();
}

/// 7-day durable outbox with crash-consistent journal semantics.
///
/// On [init], entries with stepCommitted == false are treated as partial writes
/// and removed so the next sync can retry cleanly.
class EelOfflineQueue {
  EelOfflineQueue({required OutboxStorage storage}) : _storage = storage;

  final OutboxStorage _storage;
  final Map<String, OutboxEntry> _entries = {};

  Future<void> init() async {
    final saved = await _storage.loadAll();
    for (final e in saved) {
      if (!e.stepCommitted) {
        await _storage.remove(e.id);
        continue;
      }
      if (!e.isExpired) {
        _entries[e.id] = e;
      } else {
        await _storage.remove(e.id);
      }
    }
  }

  /// Enqueue with two-phase commit (journal then commit).
  ///
  /// Phase 1: persist with stepCommitted = false.
  /// Phase 2: persist with stepCommitted = true.
  /// A crash between phases leaves stepCommitted = false; [init] removes it.
  Future<OutboxEntry> enqueue({
    required String id,
    required String idempotencyKey,
    required Map<String, Object?> payload,
  }) async {
    final entry = OutboxEntry(
      id: id,
      idempotencyKey: idempotencyKey,
      payload: payload,
      createdAt: DateTime.now(),
      stepCommitted: false,
    );
    await _storage.save(entry);
    entry.stepCommitted = true;
    await _storage.save(entry);
    _entries[id] = entry;
    return entry;
  }

  Future<void> markState(String id, OutboxEntryState state) async {
    final e = _entries[id];
    if (e == null) return;
    e.state = state;
    await _storage.save(e);
  }

  List<OutboxEntry> get pending =>
      _entries.values
          .where((e) => e.state == OutboxEntryState.pending)
          .toList();

  List<OutboxEntry> all() => _entries.values.toList();
}
