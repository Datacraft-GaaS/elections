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

/// API client: kit-sdk SPKI-pinned HTTP (Dio) with SPIRE SVID header
/// injection and Idempotency-Key generation per request.
library;


import 'package:dio/dio.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart' show EelAttestationClientStub, EelHttpClient;

import '../models/capture.dart';

class ApiClient {
  ApiClient({required this.baseUri, required this.svidProvider});

  final Uri baseUri;

  /// Supplies the SVID header, from kit-sdk boot attestation.
  final String Function() svidProvider;

  // The SVID rides on the kit-sdk client's own Dio, which is where the pinning
  // adapter, the attestation header and retry are installed.
  late final EelHttpClient _http = EelHttpClient(
    baseUrl: baseUri.toString(),
    attestation: EelAttestationClientStub(),
  )..raw.interceptors.add(_SvidInterceptor(svidProvider));

  /// POST /v1/cvr/field-notifications
  Future<SubmitCaptureResponse> submitCapture(CaptureRecord record) async {
    final idempotencyKey = _generateIdempotencyKey(record.captureId);

    final payload = {
      'nationalId': '', // populated from extracted fields in production
      'modality': _modalityToApi(record.modality),
      'officerId': record.officerId,
      'kitId': '', // populated from kit-sdk device ID
      'capturedAt': record.capturedAt.toIso8601String(),
      'geoLat': record.geoLat,
      'geoLon': record.geoLon,
      'geoAccuracyM': record.geoAccuracyM,
      'clientCertSha256': record.certSha256,
      'ocrResult': _extractedToJson(record.ocrResult),
      'ocrConfidence': record.ocrConfidence ?? 0.0,
      'officerHsmSignature': record.officerHsmSignature,
      if (record.offlineQueuedAt != null)
        'offlineQueuedAt': record.offlineQueuedAt!.toIso8601String(),
    };

    final response = await _http.post<dynamic>(
      '/v1/cvr/field-notifications',
      data: payload,
      idempotencyKey: idempotencyKey,
    );

    if (response.statusCode == 201 || response.statusCode == 200) {
      final json = (response.data as Map<String, dynamic>?) ?? const {};
      return SubmitCaptureResponse(
        captureId: json['captureId'] as String,
        eventId: json['eventId'] as String,
        queuePosition: (json['queuePosition'] as num?)?.toInt() ?? 0,
      );
    }
    // 0 means the request produced no HTTP response at all.
    throw ApiException(response.statusCode ?? 0, _bodyText(response));
  }

  /// GET /v1/cvr/field-notifications?officer_id=&status=&page=&page_size=
  Future<List<Map<String, dynamic>>> listCaptures({
    required String officerId,
    String? status,
    int page = 1,
    int pageSize = 20,
  }) async {
    final response = await _http.get<dynamic>('/v1/cvr/field-notifications',
        queryParameters: {
      'officer_id': officerId,
      if (status != null) 'status': status,
      'page': '$page',
      'page_size': '$pageSize',
    });

    if (response.statusCode == 200) {
      final json = (response.data as Map<String, dynamic>?) ?? const {};
      return (json['captures'] as List<dynamic>?)
          ?.map((e) => e as Map<String, dynamic>)
          .toList() ??
          const [];
    }
    // 0 means the request produced no HTTP response at all.
    throw ApiException(response.statusCode ?? 0, _bodyText(response));
  }

  String _generateIdempotencyKey(String captureId) {
    // Stable: keyed by captureId so retries of the same capture are idempotent.
    return 'cvr-$captureId';
  }

  String _modalityToApi(CaptureModality m) => switch (m) {
        CaptureModality.deathCertificate => 'DEATH_CERTIFICATE',
        CaptureModality.idCorrection => 'ID_CORRECTION',
      };

  Map<String, dynamic> _extractedToJson(ExtractedFields? f) {
    if (f == null) return {};
    return {
      if (f.deceasedName != null) 'deceased_name': f.deceasedName,
      if (f.nationalId != null) 'national_id': f.nationalId,
      if (f.dateOfDeath != null) 'date_of_death': f.dateOfDeath,
      if (f.placeOfDeath != null) 'place_of_death': f.placeOfDeath,
      if (f.certSerial != null) 'cert_serial': f.certSerial,
      if (f.fullName != null) 'full_name': f.fullName,
      if (f.dateOfBirth != null) 'date_of_birth': f.dateOfBirth,
      if (f.gender != null) 'gender': f.gender,
      if (f.county != null) 'county': f.county,
    };
  }

  String _bodyText(dynamic response) => response.data?.toString() ?? '';
}

/// Injects the SPIRE SVID workload identity + role headers on every request.
class _SvidInterceptor extends Interceptor {
  const _SvidInterceptor(this._svidProvider);

  final String Function() _svidProvider;

  @override
  Future<void> onRequest(
      RequestOptions options, RequestInterceptorHandler handler) async {
    options.headers['X-Spiffe-Svid'] = _svidProvider();
    options.headers['X-Eel-Role'] = 'COMMISSION_FIELD_OFFICER';
    handler.next(options);
  }
}

class SubmitCaptureResponse {
  final String captureId;
  final String eventId;
  final int queuePosition;

  const SubmitCaptureResponse({
    required this.captureId,
    required this.eventId,
    required this.queuePosition,
  });
}

class ApiException implements Exception {
  final int statusCode;
  final String body;

  const ApiException(this.statusCode, this.body);

  @override
  String toString() => 'ApiException($statusCode): $body';
}
