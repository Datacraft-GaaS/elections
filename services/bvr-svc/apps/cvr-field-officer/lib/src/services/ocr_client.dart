/// OCR client: calls py/cvr-ocr /ocr/{modality} with the captured image.
///
/// Offline fallback: when the OCR service is unreachable the raw image is
/// queued alongside the capture for server-side OCR on next sync. The capture
/// proceeds with null ocrResult / ocrConfidence in that case.
library;

import 'dart:io';
import 'dart:convert';

import 'package:http/http.dart' as http;

import '../models/capture.dart';

class OcrClient {
  final Uri baseUri;
  final Duration timeout;

  const OcrClient({
    required this.baseUri,
    this.timeout = const Duration(seconds: 30),
  });

  /// Call OCR service with [imageFile] for [modality].
  ///
  /// Returns [OcrResult] on success. Returns null and logs a warning when the
  /// service is unreachable (offline fallback; caller queues raw image).
  Future<OcrResult?> extract(File imageFile, CaptureModality modality) async {
    final modalityPath = switch (modality) {
      CaptureModality.deathCertificate => 'death-certificate',
      CaptureModality.idCorrection => 'id-correction',
    };

    final bytes = await imageFile.readAsBytes();
    final request = http.MultipartRequest(
      'POST',
      baseUri.resolve('/ocr/$modalityPath'),
    )..files.add(http.MultipartFile.fromBytes(
        'file',
        bytes,
        filename: imageFile.path.split('/').last,
      ));

    try {
      final streamed = await request.send().timeout(timeout);
      final response = await http.Response.fromStream(streamed);

      if (response.statusCode != 200) {
        return null;
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final extracted = json['extracted'] as Map<String, dynamic>? ?? {};
      return OcrResult(
        fields: ExtractedFields.fromJson(extracted),
        confidence: (json['confidence'] as num?)?.toDouble() ?? 0.0,
        adapter: json['adapter'] as String? ?? 'unknown',
        warnings: (json['warnings'] as List<dynamic>?)
                ?.map((e) => e.toString())
                .toList() ??
            [],
      );
    } on Exception {
      // Network error - offline path. Caller queues the raw image.
      return null;
    }
  }
}

class OcrResult {
  final ExtractedFields fields;
  final double confidence;
  final String adapter;
  final List<String> warnings;

  const OcrResult({
    required this.fields,
    required this.confidence,
    required this.adapter,
    required this.warnings,
  });
}
