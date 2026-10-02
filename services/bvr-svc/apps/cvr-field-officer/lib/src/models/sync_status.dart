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

library;

/// Sync status model for SyncScreen and SyncService.

enum SyncState { idle, syncing, conflict, error }

class SyncStatus {
  final SyncState state;
  final int pendingCount;
  final DateTime? lastSuccessfulSync;
  final String? errorMessage;

  const SyncStatus({
    required this.state,
    required this.pendingCount,
    this.lastSuccessfulSync,
    this.errorMessage,
  });

  const SyncStatus.idle()
      : state = SyncState.idle,
        pendingCount = 0,
        lastSuccessfulSync = null,
        errorMessage = null;

  SyncStatus copyWith({
    SyncState? state,
    int? pendingCount,
    DateTime? lastSuccessfulSync,
    String? errorMessage,
  }) {
    return SyncStatus(
      state: state ?? this.state,
      pendingCount: pendingCount ?? this.pendingCount,
      lastSuccessfulSync: lastSuccessfulSync ?? this.lastSuccessfulSync,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}
