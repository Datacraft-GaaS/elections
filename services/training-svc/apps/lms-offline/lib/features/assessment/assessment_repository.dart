import 'package:dio/dio.dart';

import '../sync/offline_queue.dart';
import 'assessment_model.dart';

class AssessmentRepository {
  final Dio _dio;
  final OfflineQueue _queue;

  AssessmentRepository({required Dio dio, required OfflineQueue queue})
      : _dio = dio,
        _queue = queue;

  Future<void> saveAttemptLocally(AssessmentAttempt attempt) async {
    final alreadyExists =
        await _queue.existsByIdempotencyKey(attempt.idempotencyKey);
    if (alreadyExists) return;

    final entry = OfflineQueueEntry(
      id: attempt.id,
      workerId: attempt.workerId,
      moduleId: attempt.moduleId,
      idempotencyKey: attempt.idempotencyKey,
      answers: attempt.answers,
      status: 'pending',
      createdAt: DateTime.now(),
    );
    await _queue.insert(entry);
  }

  Future<AssessmentAttempt> submitAttempt(AssessmentAttempt attempt) async {
    final resp = await _dio.post<Map<String, dynamic>>(
      '/v1/training/lms/assessments/${attempt.moduleId}/submissions',
      data: attempt.toJson(),
      options: Options(
        headers: {'Idempotency-Key': attempt.idempotencyKey},
      ),
    );
    final submitted = AssessmentAttempt.fromJson(resp.data!);
    await _queue.markSynced(attempt.idempotencyKey);
    return submitted;
  }

  Future<List<OfflineQueueEntry>> getPendingAttempts() {
    return _queue.getPending();
  }
}
