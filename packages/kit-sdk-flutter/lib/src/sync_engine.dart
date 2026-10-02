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
