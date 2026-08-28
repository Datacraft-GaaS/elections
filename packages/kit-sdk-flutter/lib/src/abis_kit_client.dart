import 'package:dio/dio.dart';

/// A single captured biometric template, matching the ABIS kit ingestion
/// contract (`services/abis-svc/crates/abis-api/src/routes/kit.rs`).
class KitTemplate {
  final String modality;
  final String dataB64;
  final double quality;

  const KitTemplate({
    required this.modality,
    required this.dataB64,
    required this.quality,
  });

  Map<String, dynamic> toJson() => {
        'modality': modality,
        'data_b64': dataB64,
        'quality': quality,
      };

  factory KitTemplate.fromJson(Map<String, dynamic> j) => KitTemplate(
        modality: j['modality'] as String,
        dataB64: j['data_b64'] as String,
        quality: (j['quality'] as num).toDouble(),
      );
}

class KitEnrolResult {
  final String jobId;
  final String status;
  final String subjectId;

  const KitEnrolResult({
    required this.jobId,
    required this.status,
    required this.subjectId,
  });

  factory KitEnrolResult.fromJson(Map<String, dynamic> j) => KitEnrolResult(
        jobId: j['job_id'] as String,
        status: j['status'] as String,
        subjectId: j['subject_id'] as String,
      );
}

class KitVerifyResult {
  final String subjectId;
  final bool verified;
  final double score;
  final String decision;

  const KitVerifyResult({
    required this.subjectId,
    required this.verified,
    required this.score,
    required this.decision,
  });

  factory KitVerifyResult.fromJson(Map<String, dynamic> j) => KitVerifyResult(
        subjectId: j['subject_id'] as String,
        verified: j['verified'] as bool,
        score: (j['score'] as num).toDouble(),
        decision: j['decision'] as String,
      );
}

class GalleryConfig {
  final String syncUrl;
  final DateTime? lastSyncAt;
  final int entries;

  const GalleryConfig({
    required this.syncUrl,
    this.lastSyncAt,
    required this.entries,
  });

  factory GalleryConfig.fromJson(Map<String, dynamic> j) => GalleryConfig(
        syncUrl: j['sync_url'] as String,
        lastSyncAt: j['last_sync_at'] == null
            ? null
            : DateTime.parse(j['last_sync_at'] as String),
        entries: (j['entries'] as num?)?.toInt() ?? 0,
      );
}

/// Client for the ABIS kit endpoints (integration phase P3).
///
/// - `POST /v1/kit/enrolment` — upload captured templates (ISO-19794 base64).
/// - `POST /v1/kit/verify`    — server-side 1:1 verify.
/// - `GET  /v1/kit/gallery-config` — gallery sync URL + entry count.
/// - `fetchGallery()`         — download the enrolled gallery for on-device 1:N.
///
/// Kit endpoints require `X-Kit-Attestation` and `X-SVID-Token`; both are
/// passed through when provided.
class AbisKitClient {
  AbisKitClient({
    Dio? dio,
    String? baseUrl,
    this.attestation,
    this.svidToken,
  }) : _dio = dio ??
            Dio(BaseOptions(
              baseUrl: baseUrl ??
                  const String.fromEnvironment(
                    'EEL_ABIS_URL',
                    defaultValue: 'http://localhost:8080',
                  ),
              headers: {'Accept': 'application/json'},
            ));

  final Dio _dio;
  final String? attestation;
  final String? svidToken;

  Options get _options => Options(headers: {
        if (attestation != null) 'X-Kit-Attestation': attestation!,
        if (svidToken != null) 'X-SVID-Token': svidToken!,
      });

  Future<KitEnrolResult> enrol({
    required String subjectId,
    required String operatorId,
    required String kitId,
    required List<KitTemplate> templates,
    Map<String, dynamic>? demographics,
  }) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/v1/kit/enrolment',
      options: _options,
      data: {
        'subject_id': subjectId,
        'operator_id': operatorId,
        'kit_id': kitId,
        'templates': templates.map((t) => t.toJson()).toList(),
        if (demographics != null) 'demographics': demographics,
      },
    );
    return KitEnrolResult.fromJson(res.data!);
  }

  Future<KitVerifyResult> verify({
    required String subjectId,
    required String kitId,
    required KitTemplate probeTemplate,
  }) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/v1/kit/verify',
      options: _options,
      data: {
        'subject_id': subjectId,
        'kit_id': kitId,
        'probe_template': probeTemplate.toJson(),
      },
    );
    return KitVerifyResult.fromJson(res.data!);
  }

  Future<GalleryConfig> galleryConfig() async {
    final res =
        await _dio.get<Map<String, dynamic>>('/v1/kit/gallery-config', options: _options);
    return GalleryConfig.fromJson(res.data!);
  }

  /// Download the enrolled gallery (list of templates) for on-device 1:N
  /// identify via `antIrisIdentify`.
  Future<List<KitTemplate>> fetchGallery() async {
    final config = await galleryConfig();
    final res = await _dio.get<List<dynamic>>(config.syncUrl, options: _options);
    final items = res.data ?? const [];
    return items
        .map((e) => KitTemplate.fromJson(e as Map<String, dynamic>))
        .toList();
  }
}
