import 'offline_queue.dart';

enum ConflictResolution { serverWins, localWins, userPrompt }

typedef ConflictHook = Future<ConflictResolution> Function(
    OutboxEntry local, Map<String, Object?> server);

/// Sync engine: server-wins for authoritative data, local-only for user notes.
class EelSyncEngine {
  EelSyncEngine({
    required EelOfflineQueue queue,
    ConflictHook? onConflict,
  })  : _queue = queue,
        _onConflict = onConflict;

  final EelOfflineQueue _queue;
  final ConflictHook? _onConflict;

  /// Sync pending entries. [upload] is the transport function supplied by caller.
  Future<void> sync({
    required Future<Map<String, Object?>?> Function(OutboxEntry) upload,
  }) async {
    for (final entry in _queue.pending) {
      await _queue.markState(entry.id, OutboxEntryState.syncing);
      try {
        final serverData = await upload(entry);
        if (serverData != null && _onConflict != null) {
          final resolution = await _onConflict(entry, serverData);
          if (resolution == ConflictResolution.localWins) {
            await _queue.markState(entry.id, OutboxEntryState.conflict);
            continue;
          }
        }
        await _queue.markState(entry.id, OutboxEntryState.synced);
      } catch (_) {
        await _queue.markState(entry.id, OutboxEntryState.failed);
      }
    }
  }
}
