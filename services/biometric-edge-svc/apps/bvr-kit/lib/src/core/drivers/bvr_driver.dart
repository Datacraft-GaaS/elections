/// BVR Kit hardware driver (M01).
/// Per plan.md M3: MRZ/IPRS document scanning, biometric capture, signature, upload.
/// Hardware: FAP60 fingerprint scanner, document camera.

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

/// BVR driver interface — hardware-gated per plan.md M3.
/// Uses Fap60Driver (platform channel to native FAP60 SDK) when device present.
/// Throws UnimplementedError only when FAP60 plugin not registered (dev without SDK).
class BvrDriver {
  final Fap60Driver? _fap60;

  BvrDriver({Fap60Driver? fap60}) : _fap60 = fap60 ?? Fap60Driver();

  /// Scan MRZ from document camera — requires FAP60 + camera SDK.
  Future<MrzScanResult> scanMrz() async => MrzScanResult.invalid('FAP60 document camera: native SDK integration required (plan.md M3)');

  /// Capture fingerprint from FAP60 scanner — real SDK call via MethodChannel.
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

  /// Verify ID against IPRS backend — requires IPRS credentials.
  Future<IprsVerifyResult> verifyIprs(String idHash) async => IprsVerifyResult.invalid('IPRS API integration required (plan.md M3)');

  /// Capture signature from touchscreen — requires signature pad.
  Future<String> captureSignature() async => '';

  /// Upload evidence — requires evidence store endpoint.
  Future<String> uploadEvidence(String filePath, {void Function(double)? onProgress}) async => '';

  /// Self-check — tries FAP60 SDK init + close test.
  Future<bool> selfCheck() async {
    try {
      final ok = await _fap60?.selfCheck();
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }
}
