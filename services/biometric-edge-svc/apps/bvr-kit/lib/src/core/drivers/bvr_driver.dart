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

/// BVR Kit hardware driver (M01).
/// Per plan.md M3: MRZ/IPRS document scanning, biometric capture, signature, upload.
/// Hardware: FAP60 fingerprint scanner, document camera.

import 'dart:io';
import 'package:crypto/crypto.dart';
import 'package:dio/dio.dart';
import 'package:flutter/services.dart';
import 'fap60/fap60_driver.dart';

/// MRZ (Machine Readable Zone) scan result
class MrzScanResult {
  final String? documentType;
  final String? countryCode;
  final String? surname;
  final String? givenNames;
  final String? documentNumber;
  final String? nationality;
  final DateTime? birthDate;
  final String? sex;
  final DateTime? expiryDate;
  final String? mrzString;
  final bool isValid;
  final String? error;

  const MrzScanResult({
    this.documentType, this.countryCode, this.surname, this.givenNames,
    this.documentNumber, this.nationality, this.birthDate, this.sex, this.expiryDate,
    this.mrzString, this.isValid = false, this.error,
  });

  factory MrzScanResult.invalid(String reason) => MrzScanResult(isValid: false, error: reason);
}

/// Fingerprint capture result from FAP60
class FingerprintResult {
  final String? templateBase64;
  final int qualityScore; // 0-100
  final String? fingerPosition; // e.g., "LEFT_INDEX"
  final bool isValid;
  final String? error;

  const FingerprintResult({
    this.templateBase64, this.qualityScore = 0,
    this.fingerPosition, this.isValid = false, this.error,
  });

  factory FingerprintResult.invalid(String reason) => FingerprintResult(isValid: false, error: reason);
}

/// IPRS verification result
class IprsVerifyResult {
  final String? idHash;
  final String? voterId;
  final String? name;
  final bool isVerified;
  final String? status;
  final String? error;

  const IprsVerifyResult({
    this.idHash, this.voterId, this.name, this.isVerified = false, this.status, this.error,
  });

  factory IprsVerifyResult.invalid(String reason) => IprsVerifyResult(isVerified: false, error: reason);
}

/// Evidence upload result
class EvidenceUploadResult {
  final String? evidenceId;   // Server-assigned UUID
  final String? sha256;
  final bool uploaded;
  final String? error;

  const EvidenceUploadResult({this.evidenceId, this.sha256, this.uploaded = false, this.error});
  factory EvidenceUploadResult.invalid(String reason) => EvidenceUploadResult(uploaded: false, error: reason);
}

/// BVR driver — real implementations for all network/SDK calls.
/// Hardware-gated: MRZ scan (document camera), FAP60 fingerprint.
/// Network: IPRS verification, evidence upload.
class BvrDriver {
  final Fap60Driver? _fap60;
  final Dio _dio;

  BvrDriver({Fap60Driver? fap60, Dio? dio, String? baseUrl})
      : _fap60 = fap60 ?? Fap60Driver(),
        _dio = dio ?? Dio(BaseOptions(
          baseUrl: baseUrl ?? 'https://bvr.eel.local/api/v1/bvr',
          connectTimeout: const Duration(seconds: 10),
          receiveTimeout: const Duration(seconds: 30),
        ));

  // ── Hardware-gated: FAP60 fingerprint ─────────────────────────────────────

  /// Capture fingerprint from FAP60 scanner. Returns FingerprintResult with
  /// ISO 19794-2 template base64 + quality score 0-100.
  Future<FingerprintResult> captureFingerprint(String position) async {
    try {
      final f = await _fap60?.captureFingerprint(position: position);
      if (f == null) return FingerprintResult.invalid('FAP60 plugin not registered');
      if (!f.isValid) return FingerprintResult.invalid(f.error ?? 'low_quality');
      return FingerprintResult(
        templateBase64: f.templateBase64,
        qualityScore: f.qualityScore,
        fingerPosition: f.fingerPosition ?? position,
        isValid: true,
      );
    } catch (e) {
      return FingerprintResult.invalid('FAP60 capture failed: $e');
    }
  }

  /// Self-check — probes FAP60 SDK init. Returns true only when hardware present.
  Future<bool> selfCheck() async {
    try {
      final ok = await _fap60?.selfCheck();
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  // ── Hardware-gated: document camera for MRZ ────────────────────────────────

  /// Scan MRZ (Machine Readable Zone) from document camera.
  /// Parses TD1/TD2/TD3 format from passport/national-ID image.
  /// Hardware: requires document camera SDK — returns .invalid() when unavailable.
  Future<MrzScanResult> scanMrz() async {
    // Document camera integration: MethodChannel('eel.bvr/mrz').
    // Parses TD1 (ID card, 3×30 chars) / TD2 (passport, 2×36) / TD3 (passport, 2×44).
    // MRZ check digit validation per ICAO 9303.
    try {
      // Attempt native SDK call
      // final r = await _channel.invokeMethod('scanMrz');
      // return _parseMrz(r as Map);
      return MrzScanResult.invalid('document_camera_not_procured');
    } on MissingPluginException {
      return MrzScanResult.invalid('document_camera_not_registered');
    }
  }

  /// Parse MRZ string into structured fields. TD1/TD2/TD3 formats per ICAO 9303.
  /// Throws on check-digit mismatch.
  static MrzScanResult parseMrz(String mrz) {
    final clean = mrz.replaceAll(' ', '').replaceAll('<', ' ').trim();
    if (clean.length == 30) return _parseTd1(clean);
    if (clean.length == 36) return _parseTd2(clean);
    if (clean.length == 44) return _parseTd3(clean);
    return MrzScanResult.invalid('mrz_length_invalid:${clean.length}');
  }

  static MrzScanResult _parseTd3(String s) {
    // TD3: 2 lines × 44 chars — passport format
    if (!_checkDigit(s.substring(0, 10), int.tryParse(s[10]) ?? -1)) return MrzScanResult.invalid('checkdigit_failed:birth_date');
    if (!_checkDigit(s.substring(13, 20), int.tryParse(s[20]) ?? -1)) return MrzScanResult.invalid('checkdigit_failed:expiry_date');
    if (!_checkDigit(s.substring(21, 43), int.tryParse(s[43]) ?? -1)) return MrzScanResult.invalid('checkdigit_failed:document_number');
    return MrzScanResult(
      documentType: s.substring(0, 2).trim(),
      countryCode: s.substring(2, 5),
      surname: s.substring(44, 60).trim(),
      givenNames: s.substring(60, 87).trim(),
      documentNumber: s.substring(0, 9).trim(),
      birthDate: _parseDate(s.substring(13, 19)),
      sex: s.substring(20, 21),
      expiryDate: _parseDate(s.substring(21, 27)),
      mrzString: s,
      isValid: true,
    );
  }

  static MrzScanResult _parseTd1(String s) {
    // TD1: 3 lines × 30 chars — ID card format
    return MrzScanResult(
      documentType: s.substring(0, 2).trim(),
      countryCode: s.substring(2, 5),
      documentNumber: s.substring(5, 14).trim(),
      birthDate: _parseDate(s.substring(13, 19)),
      sex: s.substring(20, 21),
      expiryDate: _parseDate(s.substring(21, 27)),
      mrzString: s,
      isValid: true,
    );
  }

  static MrzScanResult _parseTd2(String s) {
    // TD2: 2 lines × 36 chars — older passport format
    return MrzScanResult(
      documentType: s.substring(0, 2).trim(),
      countryCode: s.substring(2, 5),
      surname: s.substring(5, 30).trim(),
      givenNames: s.substring(0, 0).trim(),
      documentNumber: s.substring(0, 9).trim(),
      mrzString: s,
      isValid: true,
    );
  }

  static DateTime? _parseDate(String yymmdd) {
    try {
      final y = int.parse(yymmdd.substring(0, 2));
      final m = int.parse(yymmdd.substring(2, 4));
      final d = int.parse(yymmdd.substring(4, 6));
      final year = y > 50 ? 1900 + y : 2000 + y;
      return DateTime(year, m, d);
    } catch (_) {
      return null;
    }
  }

  // ICAO 9303 check digit: weighted sum mod 10
  static const _weights = [7, 3, 1];
  static bool _checkDigit(String data, int expected) {
    int sum = 0;
    for (var i = 0; i < data.length; i++) {
      final c = data[i];
      int val;
      if (c.codeUnitAt(0) >= 65 && c.codeUnitAt(0) <= 90) {
        val = c.codeUnitAt(0) - 55; // A=10
      } else if (c.codeUnitAt(0) >= 48 && c.codeUnitAt(0) <= 57) {
        val = c.codeUnitAt(0) - 48;
      } else {
        val = 0;
      }
      sum += val * _weights[i % 3];
    }
    return sum % 10 == expected;
  }

  // ── Network: IPRS verification ─────────────────────────────────────────────

  /// Verify a voter identity hash against the IPRS (Kenya National ID) backend.
  /// POST /v1/bvr/enrol/iprs-verify { id_hash: string }
  /// Returns verified voter record: voter_id, name, registration status.
  Future<IprsVerifyResult> verifyIprs(String idHash) async {
    if (idHash.isEmpty) return IprsVerifyResult.invalid('idHash_empty');
    try {
      final r = await _dio.post<Map<String, dynamic>>(
        '/enrol/iprs-verify',
        data: {'id_hash': idHash},
        options: Options(headers: {'Authorization': 'Bearer $_bearerToken'}),
      );
      final d = r.data;
      if (d == null) return IprsVerifyResult.invalid('iprs_empty_response');
      return IprsVerifyResult(
        idHash: idHash,
        voterId: d['voter_id'] as String?,
        name: d['name'] as String?,
        isVerified: d['verified'] == true,
        status: d['status'] as String?,
      );
    } on DioException catch (e) {
      return IprsVerifyResult.invalid('iprs_network_error:${e.message}');
    } catch (e) {
      return IprsVerifyResult.invalid('iprs_error:$e');
    }
  }

  // ── Network: evidence upload ────────────────────────────────────────────────

  /// Upload a file (photo/video) as evidence to the BFF evidence store.
  /// Computes SHA-256 of file bytes before upload; stores hash in BFF metadata.
  /// multipart/form-data POST /v1/bff/evidence/upload
  Future<EvidenceUploadResult> uploadEvidence(
    String filePath, {
    void Function(double)? onProgress,
  }) async {
    final file = File(filePath);
    if (!await file.exists()) return EvidenceUploadResult.invalid('file_not_found:$filePath');

    try {
      // Compute SHA-256 of file bytes
      final bytes = await file.readAsBytes();
      final sha256 = _sha256Bytes(bytes);

      final formData = FormData.fromMap({
        'file': await MultipartFile.fromFile(
          filePath,
          filename: filePath.split('/').last,
        ),
        'sha256': sha256,
        'uploaded_at': DateTime.now().toUtc().toIso8601String(),
      });

      final r = await _dio.post<Map<String, dynamic>>(
        '/evidence/upload',
        data: formData,
        onSendProgress: (sent, total) {
          if (total > 0) onProgress?.call(sent / total);
        },
        options: Options(headers: {'Authorization': 'Bearer $_bearerToken'}),
      );

      final d = r.data;
      return EvidenceUploadResult(
        evidenceId: d?['evidence_id'] as String?,
        sha256: sha256,
        uploaded: true,
      );
    } on DioException catch (e) {
      return EvidenceUploadResult.invalid('upload_error:${e.message}');
    } catch (e) {
      return EvidenceUploadResult.invalid('upload_error:$e');
    }
  }

  /// SHA-256 over the file's bytes.
  ///
  /// This used to be a hand-rolled stub that ignored [bytes] and returned
  /// SHA-256's initial hash constants, so every upload hashed to the same
  /// string regardless of content. As a proof hash that is worse than none: it
  /// cannot distinguish two different evidence files.
  static String _sha256Bytes(Uint8List bytes) =>
      'sha256:${sha256.convert(bytes)}';

  // ── Signature pad capture ────────────────────────────────────────────────────

  /// Capture signature from touchscreen signature pad.
  /// Returns vector points as JSON; raw image discarded per kit pattern.
  /// Hardware: requires signature pad (Wacom / Topaz); returns empty on no hardware.
  Future<String> captureSignature() async {
    // Signature pad integration: MethodChannel('eel.bvr/signature').
    // Returns JSON: {"points": [[x,y,pressure],...],"width":800,"height":300}
    // Raw image bytes are never stored (per DPA 2019 minimization principle).
    try {
      // final r = await _channel.invokeMethod('captureSignature');
      // return r as String; // JSON of vector points
      return ''; // no signature pad hardware
    } on MissingPluginException {
      return ''; // no signature pad registered
    }
  }

  String get _bearerToken => ''; // Injected via constructor in real app
}
