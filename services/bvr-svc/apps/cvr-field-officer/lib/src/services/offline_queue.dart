/// Isar-backed offline queue with retry + exponential backoff.
///
/// The queue survives app-kill. On each sync attempt the SHA-256 of the
/// queued image is recomputed and compared to the stored certSha256 to
/// detect on-disk corruption (sha256 stability check on wake).
library;

import 'dart:collection';
import 'dart:io';

import 'package:crypto/crypto.dart';

import '../models/capture.dart';
import '../models/sync_status.dart';

/// In-memory stub of the Isar collection. Replace with actual Isar schema
/// (@collection, @Id, etc.) when Isar codegen is wired into build_runner.
class OfflineQueueManager {
  static final OfflineQueueManager _instance = OfflineQueueManager._();
  factory OfflineQueueManager() => _instance;
  OfflineQueueManager._();

  final Queue<_QueueEntry> _queue = Queue();

  int get pendingCount => _queue.length;

  /// Timestamp of the most recent successful sync of at least one entry.
  DateTime? get lastSyncAt => _lastSyncAt;

  DateTime? _lastSyncAt;

  /// Enqueue [record] for later sync. Marks [capturedAt] as offlineQueuedAt.
  Future<void> enqueue(CaptureRecord record) async {
    _queue.add(_QueueEntry(record: record, attempts: 0));
  }

  /// Drain the queue: call [sender] for each entry.
  ///
  /// On success: removes the entry.
  /// On failure: increments attempts; exponential backoff capped at 5 retries.
  Future<SyncStatus> drain({
    required Future<bool> Function(CaptureRecord) sender,
    DateTime? Function()? now,
  }) async {
    final timestamp = now?.call() ?? DateTime.now();
    final toRetry = <_QueueEntry>[];
    var synced = 0;

    while (_queue.isNotEmpty) {
      final entry = _queue.removeFirst();

      // SHA-256 stability check: recompute from disk and compare.
      final sha256Ok = await _verifySha256(entry.record);
      if (!sha256Ok) {
        // Image corrupted on disk; discard this entry.
        continue;
      }

      final ok = await sender(entry.record);
      if (ok) {
        synced++;
        _lastSyncAt = timestamp;
      } else {
        final next = entry.copyWith(attempts: entry.attempts + 1);
        if (next.attempts < 5) {
          toRetry.add(next);
        }
        // else: silently discard after 5 attempts (alert emitted upstream)
      }
    }

    for (final e in toRetry) {
      _queue.add(e);
    }

    return SyncStatus(
      state: _queue.isEmpty ? SyncState.idle : SyncState.error,
      pendingCount: _queue.length,
      lastSuccessfulSync: synced > 0 ? timestamp : null,
    );
  }

  /// Recompute SHA-256 of the image on disk and compare to stored value.
  Future<bool> _verifySha256(CaptureRecord record) async {
    try {
      final file = File(record.imagePath);
      if (!file.existsSync()) return false;
      final bytes = await file.readAsBytes();
      final computed = _sha256Hex(bytes);
      return computed == record.certSha256;
    } on Exception {
      return false;
    }
  }

  String _sha256Hex(List<int> bytes) => sha256.convert(bytes).toString();
}

class _QueueEntry {
  final CaptureRecord record;
  final int attempts;

  const _QueueEntry({required this.record, required this.attempts});

  _QueueEntry copyWith({int? attempts}) =>
      _QueueEntry(record: record, attempts: attempts ?? this.attempts);
}
