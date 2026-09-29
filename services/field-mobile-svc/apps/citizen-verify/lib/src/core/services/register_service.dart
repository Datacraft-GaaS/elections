/// Register lookup service (M13) — own-record status, station, queue hints.
/// Rate limited 20/day/session per M13.

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

class RegisterService {
  final Dio _dio;
  RegisterService({Dio? dio}) : _dio = dio ?? Dio(BaseOptions(
    baseUrl: 'https://api.eel.local',
    connectTimeout: const Duration(seconds: 10),
    receiveTimeout: const Duration(seconds: 10),
  ));

  /// Look up own record by ID hash. Requires OTP-verified token.
  Future<RegisterLookup> lookup(String idHash, String otpToken) async {
    try {
      final r = await _dio.get(
        '/api/v1/mobile/bff/citizen-verify/register/${Uri.encodeComponent(idHash)}',
        options: Options(
          headers: {'Authorization': 'Bearer $otpToken'},
          // Dio rejects any status outside 2xx by throwing, which meant the
          // 429 and 404 branches below were unreachable: a rate limit and a
          // plain miss both surfaced as a raw DioException string instead of
          // the messages this service defines for them.
          validateStatus: (status) => status != null && status < 500,
        ),
      );
      if (r.statusCode == 200) {
        return RegisterLookup.fromJson(r.data);
      }
      if (r.statusCode == 429) return RegisterLookup.rateLimited;
      if (r.statusCode == 404) return RegisterLookup.notFound;
      return RegisterLookup.error('Server error ${r.statusCode}');
    } catch (e) {
      return RegisterLookup.error(e.toString());
    }
  }

  /// File objection (rectification/transfer/removal). Requires OTP-verified token.
  Future<ObjectionResult> fileObjection({
    required String idHash,
    required String otpToken,
    required String objectionType, // 'RECTIFICATION' | 'TRANSFER' | 'REMOVAL'
    required String reason,
    String? evidenceRef,
  }) async {
    try {
      final r = await _dio.post(
        '/api/v1/mobile/bff/citizen-verify/objections',
        options: Options(headers: {'Authorization': 'Bearer $otpToken'}),
        data: {
          'id_hash': idHash,
          'objection_type': objectionType,
          'reason': reason,
          'evidence_ref': evidenceRef,
          'idempotency_key': const Uuid().v7(),
        },
      );
      if (r.statusCode == 201 || r.statusCode == 200) {
        return ObjectionResult(
          ok: true,
          objectionId: r.data['objection_id'] as String?,
          status: r.data['status'] as String?,
        );
      }
      if (r.statusCode == 429) return const ObjectionResult(ok: false, error: 'Rate limited (5/day)');
      return ObjectionResult(ok: false, error: 'Server error ${r.statusCode}');
    } catch (e) {
      return ObjectionResult(ok: false, error: e.toString());
    }
  }
}

class RegisterLookup {
  final bool found;
  final String? idHash;
  final String? status; // REGISTERED, PENDING, SUSPENDED
  final String? stationCode;
  final String? stationName;
  final String? wardName;
  final String? countyName;
  final int? queueDepth;
  final String? error;

  const RegisterLookup({
    required this.found,
    this.idHash,
    this.status,
    this.stationCode,
    this.stationName,
    this.wardName,
    this.countyName,
    this.queueDepth,
    this.error,
  });

  factory RegisterLookup.fromJson(Map<String, dynamic> j) => RegisterLookup(
    found: true,
    idHash: j['id_hash'] as String?,
    status: j['status'] as String?,
    stationCode: j['station_code'] as String?,
    stationName: j['station_name'] as String?,
    wardName: j['ward_name'] as String?,
    countyName: j['county_name'] as String?,
    queueDepth: (j['queue_depth'] as num?)?.toInt(),
  );

  static const notFound = RegisterLookup(found: false);
  static const rateLimited = RegisterLookup(found: false, error: 'Rate limited (20/day)');
  factory RegisterLookup.error(String e) => RegisterLookup(found: false, error: e);
}

class ObjectionResult {
  final bool ok;
  final String? objectionId;
  final String? status;
  final String? error;
  const ObjectionResult({required this.ok, this.objectionId, this.status, this.error});
}
