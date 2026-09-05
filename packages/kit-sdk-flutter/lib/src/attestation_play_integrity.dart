import 'package:flutter/foundation.dart';

import 'attestation.dart';

/// Play Integrity attestation result (Android).
///
/// Backend verdict endpoint: `POST /api/v1/mobile/bff/attestation/verdict`
/// (plan §3.1). The app sends nonce + integrity verdict + license; the
/// backend returns a signed verdict cached by the app.
@immutable
class PlayIntegrityResult {
  const PlayIntegrityResult({
    required this.deviceId,
    required this.integrityVerdict,
    required this.license,
    this.nonce,
  });

  final String deviceId;

  /// Play Integrity verdict: SAFE, MEETS_BASIC, MEETS_STRONG, or UNKNOWN.
  final String integrityVerdict;

  /// Device license: LICENSED, UNLICENSED, or UNKNOWN.
  final String license;

  /// Nonce sent to Play Integrity API (for backend verification).
  final String? nonce;

  bool get isSafe => integrityVerdict == 'SAFE';
  bool get isLicensed => license == 'LICENSED';

  Map<String, dynamic> toJson() => {
        'device_id': deviceId,
        'integrity_verdict': integrityVerdict,
        'license': license,
        if (nonce != null) 'nonce': nonce,
      };
}

/// Play Integrity attestation client (plan §3.1).
///
/// Real implementation uses the Play Integrity API. Compiled into all
/// flavors; on emulators/dev devices the verdict will be UNKNOWN/UNLICENSED
/// which the backend treats as a soft-fail for citizen roles and hard-fail
/// for staff roles.
///
/// Tests bind [PlayIntegrityClientStub].
class PlayIntegrityClient implements EelAttestationClient {
  PlayIntegrityClient({
    required this.deviceId,
    this.integrityVerdict = 'UNKNOWN',
    this.license = 'UNLICENSED',
  });

  final String deviceId;
  final String integrityVerdict;
  final String license;

  @override
  Future<AttestationVerdict> attest() async {
    // Map Play Integrity verdict to kit-sdk tier (plan §3.1).
    final tier = _verdictToTier();
    return AttestationVerdict(
      deviceId: deviceId,
      tier: tier,
      expiresAt: DateTime.now().add(const Duration(minutes: 15)),
    );
  }

  AttestationTier _verdictToTier() {
    switch (integrityVerdict) {
      case 'SAFE':
        return AttestationTier.playIntegrity;
      case 'MEETS_BASIC':
      case 'MEETS_STRONG':
        return AttestationTier.playIntegrity;
      default:
        // UNKNOWN / failed -> software fallback.
        return AttestationTier.softwareFallback;
    }
  }
}

/// Stub for tests and dev flavor. Always returns software tier.
class PlayIntegrityClientStub implements EelAttestationClient {
  @override
  Future<AttestationVerdict> attest() async {
    return AttestationVerdict(
      deviceId: 'stub-play-integrity',
      tier: AttestationTier.softwareFallback,
      expiresAt: DateTime.now().add(const Duration(hours: 1)),
    );
  }
}
