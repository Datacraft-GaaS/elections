/// OTP service — M13/M29 FR-C2.
/// Phone number → SMS OTP → JWT (rate-limited 3 attempts / 10 min).
/// OTP verified → citizen can access register lookup + objection filing.
/// Per plan.md §781: OTP auth on public path, rate limited 20/day/session.

import 'package:dio/dio.dart';

class OtpService {
  final Dio _dio;
  OtpService({Dio? dio}) : _dio = dio ?? Dio(BaseOptions(
    baseUrl: 'https://api.eel.local',
    connectTimeout: const Duration(seconds: 10),
    receiveTimeout: const Duration(seconds: 10),
  ));

  /// Send OTP to phone number. Returns true on success.
  Future<bool> sendOtp(String phoneNumber) async {
    try {
      final r = await _dio.post('/api/v1/mobile/bff/citizen-verify/otp/send', data: {
        'phone': phoneNumber,
        'idempotency_key': _uuid7(),
      });
      return r.statusCode == 200 || r.statusCode == 201;
    } catch (_) { return false; }
  }

  /// Verify OTP. Returns JWT on success.
  Future<OtpVerifyResult> verifyOtp(String phoneNumber, String code) async {
    try {
      final r = await _dio.post('/api/v1/mobile/bff/citizen-verify/otp/verify', data: {
        'phone': phoneNumber,
        'code': code,
        'idempotency_key': _uuid7(),
      });
      if (r.statusCode == 200 || r.statusCode == 201) {
        return OtpVerifyResult(
          ok: true,
          token: r.data['token'] as String?,
          expiresIn: (r.data['expires_in'] as num?)?.toInt(),
        );
      }
      if (r.statusCode == 401) return OtpVerifyResult(ok: false, error: 'Invalid OTP');
      if (r.statusCode == 429) return OtpVerifyResult(ok: false, error: 'Too many attempts');
      return OtpVerifyResult(ok: false, error: 'Server error ${r.statusCode}');
    } catch (e) {
      return OtpVerifyResult(ok: false, error: e.toString());
    }
  }

  String _uuid7() {
    final now = DateTime.now().microsecondsSinceEpoch;
    final random = DateTime.now().millisecondsSinceEpoch % 0xFFFFFFFFFFFF;
    return '${(now ~/ 1000).toRadixString(16)}-${(random.toRadixString(16).padLeft(10, '0'))}';
  }
}

class OtpVerifyResult {
  final bool ok;
  final String? token;
  final int? expiresIn;
  final String? error;
  const OtpVerifyResult({required this.ok, this.token, this.expiresIn, this.error});
}
