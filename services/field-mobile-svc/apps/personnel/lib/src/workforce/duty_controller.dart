import 'package:dio/dio.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'workforce_api.dart';

/// Raised for client-side validation before any network call.
class DutyValidationException implements Exception {
  final String message;
  const DutyValidationException(this.message);

  @override
  String toString() => message;
}

class DutyState {
  final List<WorkforcePosting> postings;
  final Map<String, List<WorkforceTask>> tasksByPosting;
  final bool loading;
  final String? errorMessage;

  const DutyState({
    this.postings = const [],
    this.tasksByPosting = const {},
    this.loading = false,
    this.errorMessage,
  });

  bool get hasCachedData => postings.isNotEmpty;

  DutyState copyWith({
    List<WorkforcePosting>? postings,
    Map<String, List<WorkforceTask>>? tasksByPosting,
    bool? loading,
    String? errorMessage,
  }) =>
      DutyState(
        postings: postings ?? this.postings,
        tasksByPosting: tasksByPosting ?? this.tasksByPosting,
        loading: loading ?? this.loading,
        // Keep last error until explicitly cleared with null.
        errorMessage: errorMessage == _errorSentinel
            ? null
            : (errorMessage ?? this.errorMessage),
      );

  static const _errorSentinel = '__clear__';
  static const clearError = _errorSentinel;
}

/// Offline-first controller for the My Duty screen (design §6, §9).
///
/// Reads are cached in memory; ack/done apply optimistically and go through
/// workforce-svc as a write-through. On failure the mutation is enqueued in
/// the EelOfflineQueue outbox and drained later by [sync].
class DutyController extends ChangeNotifier {
  DutyController({
    WorkforceApi? api,
    EelOfflineQueue? queue,
    EelSyncEngine? syncEngine,
    String workerId = 'local-worker',
  })  : _api = api ?? WorkforceApi(),
        _queue = queue ?? EelOfflineQueue(storage: InMemoryOutboxStorage()),
        _syncEngine = syncEngine,
        _workerId = workerId {
    _ready = _queue.init();
  }

  static const actionAck = 'task.ack';
  static const actionDone = 'task.done';

  final WorkforceApi _api;
  final EelOfflineQueue _queue;
  final EelSyncEngine? _syncEngine;
  final String _workerId;

  late final Future<void> _ready;
  int _seq = 0;

  DutyState state = const DutyState();

  Future<void> refresh() async {
    await _ready;
    state = state.copyWith(loading: true);
    notifyListeners();
    try {
      final postings = await _api.listActivePostings(_workerId);
      final tasks = <String, List<WorkforceTask>>{};
      for (final p in postings) {
        tasks[p.postingId] = _overlayPending(
          await _api.listTasks(p.postingId),
        );
      }
      state = state.copyWith(
        postings: postings,
        tasksByPosting: tasks,
        loading: false,
        errorMessage: DutyState.clearError,
      );
    } catch (e) {
      // Offline resilience: keep cached data, surface the failure.
      state = state.copyWith(
        loading: false,
        errorMessage: describeDutyError(e),
      );
    }
    notifyListeners();
  }

  /// Local-first ack: optimistic status flip, write-through, enqueue on failure.
  Future<void> ackTask(WorkforceTask task) async {
    await _mutate(task, actionAck, evidenceHash: null);
  }

  /// Local-first done. Critical tasks require an evidence hash; validation
  /// fails before any network call.
  Future<void> doneTask(WorkforceTask task, String? evidenceHash) async {
    if (task.critical && (evidenceHash == null || evidenceHash.isEmpty)) {
      throw const DutyValidationException(
          'Evidence hash is required for critical tasks');
    }
    await _mutate(task, actionDone, evidenceHash: evidenceHash);
  }

  Future<void> _mutate(
    WorkforceTask task,
    String action, {
    required String? evidenceHash,
  }) async {
    await _ready;
    _applyLocal(task.assignmentId, action, evidenceHash);
    try {
      await _send(action, task.assignmentId, evidenceHash);
    } catch (_) {
      await _enqueue(action, task.assignmentId, evidenceHash);
    }
    notifyListeners();
  }

  void _applyLocal(String taskId, String action, String? evidenceHash) {
    final updated = <String, List<WorkforceTask>>{};
    state.tasksByPosting.forEach((postingId, tasks) {
      updated[postingId] = [
        for (final t in tasks)
          if (t.assignmentId == taskId)
            t.copyWith(
              status: action == actionDone ? TaskStatus.done : TaskStatus.acked,
              evidenceHash:
                  action == actionDone ? evidenceHash : t.evidenceHash,
            )
          else
            t,
      ];
    });
    state = state.copyWith(tasksByPosting: updated);
  }

  Future<OutboxEntry> _enqueue(
    String action,
    String taskId,
    String? evidenceHash,
  ) async {
    _seq += 1;
    return _queue.enqueue(
      id: '$action-$taskId-$_seq',
      idempotencyKey: '$action:$taskId:${DateTime.now().microsecondsSinceEpoch}',
      payload: {
        'action': action,
        'taskId': taskId,
        if (evidenceHash != null) 'evidenceHash': evidenceHash,
      },
    );
  }

  Future<void> _send(String action, String taskId, String? evidenceHash) async {
    switch (action) {
      case actionAck:
        await _api.ackTask(taskId);
      case actionDone:
        await _api.doneTask(taskId, evidenceHash!);
      default:
        throw StateError('unknown duty action: $action');
    }
  }

  /// Replays pending outbox mutations over freshly fetched server tasks so
  /// unsynced local actions survive a refresh/restart until synced.
  List<WorkforceTask> _overlayPending(List<WorkforceTask> fetched) {
    final pending = {for (final e in _queue.pending) e.id: e};
    if (pending.isEmpty) return fetched;
    return [
      for (final t in fetched)
        () {
          OutboxEntry? match;
          for (final e in pending.values) {
            if (e.payload['taskId'] == t.assignmentId &&
                ((e.payload['action'] == actionDone &&
                        t.status == TaskStatus.done) ||
                    (e.payload['action'] == actionAck))) {
              match = e;
              break;
            }
          }
          if (match == null) return t;
          final isDone = match.payload['action'] == actionDone;
          return t.copyWith(
            status: isDone ? TaskStatus.done : TaskStatus.acked,
            evidenceHash:
                isDone ? match.payload['evidenceHash'] as String? : null,
          );
        }(),
    ];
  }

  /// Drains the outbox via EelSyncEngine (or inline fallback), one API call
  /// per pending entry; failures mark entries failed for retry.
  Future<void> sync() async {
    await _ready;
    final engine = _syncEngine ??
        EelSyncEngine(queue: _queue); // test/inline fallback
    await engine.sync(upload: (entry) async {
      final action = entry.payload['action'] as String;
      final taskId = entry.payload['taskId'] as String;
      await _send(action, taskId, entry.payload['evidenceHash'] as String?);
      return null;
    });
  }

  List<OutboxEntry> get pendingEntries => _queue.pending;
}

/// Extracts human-readable server error text from an exception.
String describeDutyError(Object error) {
  if (error is DioException) {
    final data = error.response?.data;
    if (data is Map && data['error'] != null) return data['error'].toString();
    if (data is String && data.isNotEmpty) return data;
    return error.message ?? 'network error';
  }
  return error.toString();
}

final dutyApiProvider =
    Provider<WorkforceApi>((ref) => WorkforceApi());

final dutyQueueProvider = Provider<EelOfflineQueue>(
  (ref) => EelOfflineQueue(storage: InMemoryOutboxStorage()),
);

final dutySyncEngineProvider = Provider<EelSyncEngine>(
  (ref) => EelSyncEngine(queue: ref.watch(dutyQueueProvider)),
);

final dutyControllerProvider = ChangeNotifierProvider<DutyController>((ref) {
  return DutyController(
    api: ref.watch(dutyApiProvider),
    queue: ref.watch(dutyQueueProvider),
    syncEngine: ref.watch(dutySyncEngineProvider),
  );
});
