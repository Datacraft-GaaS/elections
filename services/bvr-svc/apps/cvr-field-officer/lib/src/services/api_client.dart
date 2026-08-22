/// API client: HTTP client with SPIRE SVID header injection and
/// Idempotency-Key generation per request.
library;

import 'dart:convert';

import 'package:http/http.dart' as http;

import '../models/capture.dart';

class ApiClient {
  final Uri baseUri;
  final String Function() svidProvider; // provided by kit-sdk boot attestation

  const ApiClient({
    required this.baseUri,
    required this.svidProvider,
  });

  /// POST /v1/cvr/field-notifications
  Future<SubmitCaptureResponse> submitCapture(CaptureRecord record) async {
    final idempotencyKey = _generateIdempotencyKey(record.captureId);
    final svid = svidProvider();

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

    final response = await http.post(
      baseUri.resolve('/v1/cvr/field-notifications'),
      headers: {
        'Content-Type': 'application/json',
        'X-Spiffe-Svid': svid,
        'X-Eel-Role': 'COMMISSION_FIELD_OFFICER',
        'Idempotency-Key': idempotencyKey,
      },
      body: jsonEncode(payload),
    );

    if (response.statusCode == 201 || response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      return SubmitCaptureResponse(
        captureId: json['captureId'] as String,
        eventId: json['eventId'] as String,
        queuePosition: (json['queuePosition'] as num?)?.toInt() ?? 0,
      );
    }
    throw ApiException(response.statusCode, response.body);
  }

  /// GET /v1/cvr/field-notifications?officer_id=&status=&page=&page_size=
  Future<List<Map<String, dynamic>>> listCaptures({
    required String officerId,
    String? status,
    int page = 1,
    int pageSize = 20,
  }) async {
    final svid = svidProvider();
    final uri = baseUri.resolve('/v1/cvr/field-notifications').replace(
      queryParameters: {
        'officer_id': officerId,
        if (status != null) 'status': status,
        'page': '$page',
        'page_size': '$pageSize',
      },
    );

    final response = await http.get(uri, headers: {
      'X-Spiffe-Svid': svid,
      'X-Eel-Role': 'COMMISSION_FIELD_OFFICER',
    });

    if (response.statusCode == 200) {
      final json = jsonDecode(response.body) as Map<String, dynamic>;
      return (json['captures'] as List<dynamic>)
          .map((e) => e as Map<String, dynamic>)
          .toList();
    }
    throw ApiException(response.statusCode, response.body);
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
