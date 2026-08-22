/// Isar-backed offline queue with 7-day capacity and crash-safe WAL journal.
///
/// Signals are accepted synchronously (in-memory enqueue) then persisted to
/// Isar.  On reconnect, [SinkDispatcher] drains the queue in FIFO order with
/// exponential-backoff retry.
library;

import 'dart:collection';

import 'signal_sink.dart';

/// Persistence contract — swapped for Isar in production, in-memory in tests.
abstract class TelemetryQueueStorage {
  Future<void> persist(TelemetryQueueEntry entry);
  Future<void> remove(String id);
  Future<List<TelemetryQueueEntry>> loadAll();
  Future<void> clear();
}

/// Serialisable wrapper around a [Signal] for queue storage.
class TelemetryQueueEntry {
  TelemetryQueueEntry({
    required this.id,
    required this.enqueuedAt,
    required this.signalJson,
    this.attempts = 0,
    this.nextRetryAt,
  });

  final String id;
  final DateTime enqueuedAt;
  final Map<String, Object?> signalJson;
  int attempts;
  DateTime? nextRetryAt;

  bool get isExpired =>
      DateTime.now().difference(enqueuedAt) > const Duration(days: 7);

  Map<String, Object?> toJson() => {
        'id': id,
        'enqueuedAt': enqueuedAt.toUtc().toIso8601String(),
        'signalJson': signalJson,
        'attempts': attempts,
        'nextRetryAt': nextRetryAt?.toUtc().toIso8601String(),
      };

  factory TelemetryQueueEntry.fromJson(Map<String, Object?> j) =>
      TelemetryQueueEntry(
        id: j['id'] as String,
        enqueuedAt: DateTime.parse(j['enqueuedAt'] as String),
        signalJson: (j['signalJson'] as Map).cast<String, Object?>(),
        attempts: j['attempts'] as int? ?? 0,
        nextRetryAt: (j['nextRetryAt'] as String?) != null
            ? DateTime.parse(j['nextRetryAt'] as String)
            : null,
      );
}

/// In-memory storage — used by tests and as a build-time stub.
class InMemoryTelemetryStorage implements TelemetryQueueStorage {
  final _store = <String, TelemetryQueueEntry>{};

  @override
  Future<void> persist(TelemetryQueueEntry entry) async =>
      _store[entry.id] = entry;

  @override
  Future<void> remove(String id) async => _store.remove(id);

  @override
  Future<List<TelemetryQueueEntry>> loadAll() async =>
      _store.values.toList(growable: false);

  @override
  Future<void> clear() async => _store.clear();
}

/// Offline queue — accepts [Signal]s, persists them, exposes drain iterator.
class TelemetryOfflineQueue implements SignalSink {
  TelemetryOfflineQueue({required TelemetryQueueStorage storage})
      : _storage = storage;

  final TelemetryQueueStorage _storage;
  final _queue = Queue<TelemetryQueueEntry>();
  int _counter = 0;

  /// Load persisted entries from storage on startup (crash recovery).
  Future<void> init() async {
    final entries = await _storage.loadAll();
    // Sort by enqueuedAt for FIFO order.
    entries.sort((a, b) => a.enqueuedAt.compareTo(b.enqueuedAt));
    for (final e in entries) {
      if (!e.isExpired) _queue.addLast(e);
    }
    // Purge expired entries.
    final expired = entries.where((e) => e.isExpired).toList();
    for (final e in expired) {
      await _storage.remove(e.id);
    }
  }

  @override
  Future<void> accept(Signal signal) async {
    final entry = TelemetryQueueEntry(
      id: '${signal.deviceId}-${_counter++}-${signal.monotonicCounter}',
      enqueuedAt: DateTime.now().toUtc(),
      signalJson: signal.toJson(),
    );
    _queue.addLast(entry);
    await _storage.persist(entry);
  }

  @override
  Future<void> flush() async {
    // Flushing is delegated to SinkDispatcher which drains via [drainBatch].
  }

  @override
  Future<void> close() async {}

  /// Returns up to [maxSize] pending entries for dispatch.
  List<TelemetryQueueEntry> drainBatch({int maxSize = 1000}) {
    final batch = <TelemetryQueueEntry>[];
    for (final entry in _queue) {
      if (batch.length >= maxSize) break;
      final now = DateTime.now().toUtc();
      if (entry.nextRetryAt == null || now.isAfter(entry.nextRetryAt!)) {
        batch.add(entry);
      }
    }
    return batch;
  }

  /// Acknowledges successful dispatch and removes from queue + storage.
  Future<void> acknowledge(List<String> ids) async {
    _queue.removeWhere((e) => ids.contains(e.id));
    for (final id in ids) {
      await _storage.remove(id);
    }
  }

  /// Marks entries for retry with exponential backoff (base 5 s, max 5 min).
  Future<void> scheduleRetry(List<String> ids) async {
    for (final entry in _queue) {
      if (!ids.contains(entry.id)) continue;
      entry.attempts++;
      final backoffSec = (5 * (1 << entry.attempts.clamp(0, 6))).clamp(5, 300);
      entry.nextRetryAt =
          DateTime.now().toUtc().add(Duration(seconds: backoffSec));
      await _storage.persist(entry);
    }
  }

  int get pendingCount => _queue.length;
}
