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

/// Offline queue integration for rts-kit.
///
/// Wraps [EelOfflineQueue] from eel_kit_sdk with a SQLite-backed
/// [OutboxStorage] so queue entries survive process restarts and power cuts.
///
/// The two-phase commit in [EelOfflineQueue.enqueue] guarantees that an entry
/// with stepCommitted=false is removed on next [EelOfflineQueue.init], so a
/// mid-write power cut never leaves a phantom entry that would be re-transmitted
/// with stale data.
///
/// Retry schedule: pending entries older than 60 s are re-attempted by
/// [RtsQueueProcessor] with exponential back-off (1s, 2s, 4s, cap 30s).
/// Entries expire after 7 days (enforced by [OutboxEntry.isExpired]).
library;

import 'dart:convert';

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';
import 'package:path/path.dart' as p;
import 'package:sqflite/sqflite.dart';

final _log = Logger('eel.rts.queue');

// ---- SQLite storage ---------------------------------------------------------

class SqliteOutboxStorage implements OutboxStorage {
  static const _kTableName = 'outbox';
  static const _kDbName = 'eel_rts_outbox.db';

  Database? _db;

  Future<void> open() async {
    final dbPath = p.join(await getDatabasesPath(), _kDbName);
    _db = await openDatabase(
      dbPath,
      version: 1,
      onCreate: (db, version) => db.execute('''
        CREATE TABLE $_kTableName (
          id TEXT PRIMARY KEY,
          data TEXT NOT NULL
        )
      '''),
    );
    _log.info('Opened outbox DB at $dbPath');
  }

  Database get _database {
    final db = _db;
    assert(db != null, 'SqliteOutboxStorage.open() must be called before use');
    return db!;
  }

  @override
  Future<void> save(OutboxEntry entry) async {
    await _database.insert(
      _kTableName,
      {'id': entry.id, 'data': jsonEncode(entry.toJson())},
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  @override
  Future<void> remove(String id) async {
    await _database.delete(_kTableName, where: 'id = ?', whereArgs: [id]);
  }

  @override
  Future<List<OutboxEntry>> loadAll() async {
    final rows = await _database.query(_kTableName);
    return rows.map((r) {
      final data = jsonDecode(r['data'] as String) as Map<String, Object?>;
      return OutboxEntry.fromJson(data);
    }).toList();
  }

  Future<void> close() async => _db?.close();
}

// ---- Providers --------------------------------------------------------------

/// Global queue provider. Overridden in [main] after [EelOfflineQueue.init].
final outboxQueueProvider = Provider<EelOfflineQueue>(
  (ref) => throw UnimplementedError('outboxQueueProvider must be overridden'),
);

// ---- Retry processor --------------------------------------------------------

/// Drives queue retries. Call [dispose] when the app goes to background.
class RtsQueueProcessor {
  RtsQueueProcessor({required EelOfflineQueue queue}) : _queue = queue;

  final EelOfflineQueue _queue;
  bool _running = false;

  void start(Future<void> Function(OutboxEntry) sender) {
    if (_running) return;
    _running = true;
    _loop(sender);
  }

  void stop() => _running = false;

  Future<void> _loop(Future<void> Function(OutboxEntry) sender) async {
    var backoff = const Duration(seconds: 1);
    while (_running) {
      final pending = _queue.pending;
      for (final entry in pending) {
        if (!_running) break;
        try {
          await _queue.markState(entry.id, OutboxEntryState.syncing);
          await sender(entry);
          await _queue.markState(entry.id, OutboxEntryState.synced);
          backoff = const Duration(seconds: 1);
          _log.info('Queue retry success id=${entry.id}');
        } catch (e) {
          await _queue.markState(entry.id, OutboxEntryState.pending);
          _log.warning('Queue retry failed id=${entry.id}: $e');
          backoff = Duration(
            seconds: (backoff.inSeconds * 2).clamp(1, 30),
          );
        }
      }
      await Future<void>.delayed(backoff);
    }
  }
}
