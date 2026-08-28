import 'package:dio/dio.dart';

import '../models/worker_assignment.dart';
import '../models/worker_check_in.dart';
import '../models/worker_check_out.dart';
import '../models/worker_status_ping.dart';

/// Base URL for the personnel dispatch API, overridable via --dart-define.
const String kDispatchBaseUrl = String.fromEnvironment(
  'EEL_DISPATCH_URL',
  defaultValue: 'http://localhost:8083',
);

/// Typed client over the dispatch-api (`/v1/dispatch/*`, camelCase JSON).
class DispatchApi {
  DispatchApi({Dio? dio, String? baseUrl})
      : _dio = dio ??
            Dio(BaseOptions(
              baseUrl: baseUrl ?? kDispatchBaseUrl,
              connectTimeout: const Duration(seconds: 10),
              receiveTimeout: const Duration(seconds: 30),
            ));

  final Dio _dio;

  Future<WorkerCheckIn> checkIn({
    required String workerId,
    required String stationCode,
    required String electionId,
    required String biometricHash,
    required double gpsLat,
    required double gpsLon,
    required double gpsAccuracyM,
    required String signature,
  }) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/v1/dispatch/check-in',
      data: {
        'workerId': workerId,
        'stationCode': stationCode,
        'electionId': electionId,
        'biometricHash': biometricHash,
        'gpsLat': gpsLat,
        'gpsLon': gpsLon,
        'gpsAccuracyM': gpsAccuracyM,
        'signature': signature,
      },
    );
    return WorkerCheckIn.fromJson(res.data!);
  }

  Future<WorkerCheckOut> checkOut({
    required String workerId,
    required String stationCode,
    required String electionId,
    required double gpsLat,
    required double gpsLon,
    required String signature,
  }) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/v1/dispatch/check-out',
      data: {
        'workerId': workerId,
        'stationCode': stationCode,
        'electionId': electionId,
        'gpsLat': gpsLat,
        'gpsLon': gpsLon,
        'signature': signature,
      },
    );
    return WorkerCheckOut.fromJson(res.data!);
  }

  Future<WorkerStatusPing> ping({
    required String workerId,
    required String stationCode,
    required WorkerStatus status,
    String? reason,
  }) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/v1/dispatch/ping',
      data: {
        'workerId': workerId,
        'stationCode': stationCode,
        'status': status.apiValue,
        if (reason != null) 'reason': reason,
      },
    );
    return WorkerStatusPing.fromJson(res.data!);
  }

  /// Latest recorded status for a worker, or `null` when there is none yet.
  Future<WorkerStatusPing?> getStatus(String workerId) async {
    try {
      final res = await _dio.get<Map<String, dynamic>>(
        '/v1/dispatch/workers/$workerId/status',
      );
      return WorkerStatusPing.fromJson(res.data!);
    } on DioException catch (e) {
      if (e.response?.statusCode == 404) return null;
      rethrow;
    }
  }

  Future<List<WorkerAssignment>> getAssignments(String workerId) async {
    final res = await _dio.get<List<dynamic>>(
      '/v1/dispatch/assignments/$workerId',
    );
    final items = res.data ?? const [];
    return items
        .map((e) => WorkerAssignment.fromJson(e as Map<String, dynamic>))
        .toList();
  }
}
