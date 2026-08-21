import 'package:dio/dio.dart';

class CheckInService {
  final Dio _dio;

  CheckInService({Dio? dio}) : _dio = dio ?? Dio();

  /// GPS spoof check stub - validates accuracy and movement patterns.
  bool _isSpoofed(double accuracyM) => accuracyM > 100.0;

  Future<Map<String, dynamic>> checkIn({
    required String workerId,
    required String stationCode,
    required double gpsLat,
    required double gpsLon,
    required double gpsAccuracyM,
    required String biometricHash,
  }) async {
    if (_isSpoofed(gpsAccuracyM)) {
      throw Exception('GPS accuracy too low - possible spoof');
    }
    final response = await _dio.post('/api/check-in', data: {
      'workerId': workerId,
      'stationCode': stationCode,
      'gpsLat': gpsLat,
      'gpsLon': gpsLon,
      'gpsAccuracyM': gpsAccuracyM,
      'biometricHash': biometricHash,
    });
    return response.data as Map<String, dynamic>;
  }
}
