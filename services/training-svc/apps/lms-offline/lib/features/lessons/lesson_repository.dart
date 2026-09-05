import 'dart:convert';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:dio/dio.dart';
import '../sync/offline_queue.dart';

class LessonRepository {
  final Dio _dio;
  final OfflineQueue _queue;
  static const _kCacheLimit = 20;
  LessonRepository({required Dio dio, required OfflineQueue queue})
      : _dio = dio, _queue = queue;
  Future<bool> _isOffline() async {
    final r = await Connectivity().checkConnectivity();
    return r.contains(ConnectivityResult.none) || r.isEmpty;
  }
  Future<List<Map<String, dynamic>>> fetchLessons(String moduleId) async {
    final resp = await _dio.get<List<dynamic>>('/v1/training/lms/modules/$moduleId/lessons');
    return resp.data?.cast<Map<String, dynamic>>() ?? [];
  }
  Future<void> cacheLesson(String lessonId) async {
    final db = await _queue.rawDb;
    final cached = await db.query('cached_lessons', where: 'id = ?', whereArgs: [lessonId]);
    if (cached.isNotEmpty) return;
    final resp = await _dio.get<Map<String, dynamic>>('/v1/training/lms/lessons/$lessonId');
    final lesson = resp.data!;
    await db.insert('cached_lessons', {...lesson, 'cached_at': DateTime.now().toIso8601String()});
    final all = await db.query('cached_lessons', orderBy: 'cached_at ASC');
    if (all.length > _kCacheLimit) {
      for (final row in all.take(all.length - _kCacheLimit)) {
        await db.delete('cached_lessons', where: 'id = ?', whereArgs: [row['id']]);
      }
    }
  }
  Future<List<Map<String, dynamic>>> getCachedLessons(String moduleId) async {
    final db = await _queue.rawDb;
    return db.query('cached_lessons', orderBy: 'order_index ASC');
  }
  Future<void> completeLesson(String workerId, String lessonId, bool completed, {String? idempotencyKey}) async {
    final key = idempotencyKey ?? '$workerId-$lessonId-completed';
    final body = {'worker_id': workerId, 'lesson_id': lessonId, 'completed': completed, 'idempotency_key': key};
    if (await _isOffline()) {
      await _queue.insert(OfflineQueueEntry(id: key, workerId: workerId, moduleId: lessonId, idempotencyKey: key, answers: body, status: 'pending', createdAt: DateTime.now()));
      return;
    }
    await _dio.post('/v1/training/lms/lessons/$lessonId/complete', data: body, options: Options(headers: {'Idempotency-Key': key}));
    await _queue.markSynced(key);
  }
}
