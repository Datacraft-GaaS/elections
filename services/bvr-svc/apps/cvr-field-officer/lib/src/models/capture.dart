library;

/// Data models for CVR field-officer captures.

enum CaptureModality { deathCertificate, idCorrection }

enum ReviewVerdict { pending, accepted, rejected, manualReview }

/// Local capture record (stored in Isar offline queue before sync).
class CaptureRecord {
  final String captureId; // UUID7 string, generated locally
  final String officerId;
  final CaptureModality modality;
  final DateTime capturedAt;
  final double geoLat;
  final double geoLon;
  final double geoAccuracyM;
  final String imagePath; // local file path
  final String certSha256; // hex SHA-256 of image bytes
  final ExtractedFields? ocrResult;
  final double? ocrConfidence;
  final String officerHsmSignature; // hex Ed25519 sig from kit-sdk attestation
  final DateTime? offlineQueuedAt;
  final ReviewVerdict verdict;

  const CaptureRecord({
    required this.captureId,
    required this.officerId,
    required this.modality,
    required this.capturedAt,
    required this.geoLat,
    required this.geoLon,
    required this.geoAccuracyM,
    required this.imagePath,
    required this.certSha256,
    this.ocrResult,
    this.ocrConfidence,
    required this.officerHsmSignature,
    this.offlineQueuedAt,
    this.verdict = ReviewVerdict.pending,
  });

  CaptureRecord copyWith({
    ExtractedFields? ocrResult,
    double? ocrConfidence,
    ReviewVerdict? verdict,
  }) {
    return CaptureRecord(
      captureId: captureId,
      officerId: officerId,
      modality: modality,
      capturedAt: capturedAt,
      geoLat: geoLat,
      geoLon: geoLon,
      geoAccuracyM: geoAccuracyM,
      imagePath: imagePath,
      certSha256: certSha256,
      ocrResult: ocrResult ?? this.ocrResult,
      ocrConfidence: ocrConfidence ?? this.ocrConfidence,
      officerHsmSignature: officerHsmSignature,
      offlineQueuedAt: offlineQueuedAt,
      verdict: verdict ?? this.verdict,
    );
  }
}

/// Extracted fields returned by py/cvr-ocr.
class ExtractedFields {
  // Death certificate
  final String? deceasedName;
  final String? nationalId;
  final String? dateOfDeath;
  final String? placeOfDeath;
  final String? certSerial;

  // ID correction
  final String? fullName;
  final String? dateOfBirth;
  final String? gender;
  final String? county;

  const ExtractedFields({
    this.deceasedName,
    this.nationalId,
    this.dateOfDeath,
    this.placeOfDeath,
    this.certSerial,
    this.fullName,
    this.dateOfBirth,
    this.gender,
    this.county,
  });

  factory ExtractedFields.fromJson(Map<String, dynamic> json) {
    return ExtractedFields(
      deceasedName: json['deceased_name'] as String?,
      nationalId: json['national_id'] as String?,
      dateOfDeath: json['date_of_death'] as String?,
      placeOfDeath: json['place_of_death'] as String?,
      certSerial: json['cert_serial'] as String?,
      fullName: json['full_name'] as String?,
      dateOfBirth: json['date_of_birth'] as String?,
      gender: json['gender'] as String?,
      county: json['county'] as String?,
    );
  }
}
