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
