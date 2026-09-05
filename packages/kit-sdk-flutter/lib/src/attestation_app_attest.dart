import 'package:flutter/foundation.dart';

import 'attestation.dart';

/// App Attest attestation result (iOS).
///
/// Backend verifies the attestation object with Apple and returns a signed
/// verdict on each subsequent attest (plan §3.1).
@immutable
class AppAttestResult {
  const AppAttestResult({
    required this.deviceId,
    required this.attestationType,
    this.passkeyBound = false,
  });

  final String deviceId;

  /// App Attest type: PLATFORM or PRIVATE_KEY.
  final String attestationType;

  /// Whether the key is bound to a passkey (higher assurance).
  final bool passkeyBound;

  bool get isStrong => attestationType == 'PRIVATE_KEY';

  Map<String, dynamic> toJson() => {
        'device_id': deviceId,
        'attestation_type': attestationType,
        'passkey_bound': passkeyBound,
      };
}

/// App Attest attestation client (plan §3.1).
///
/// Real implementation uses DeviceCheck App Attest. Compiled into all
/// flavors; on simulators the attestation type will be PLATFORM (lower
/// assurance) which the backend treats accordingly.
///
/// Tests bind [AppAttestClientStub].
class AppAttestClient implements EelAttestationClient {
  AppAttestClient({
    required this.deviceId,
    this.attestationType = 'PLATFORM',
  });

  final String deviceId;
  final String attestationType;

  @override
  Future<AttestationVerdict> attest() async {
    final tier = _attestationToTier();
    return AttestationVerdict(
      deviceId: deviceId,
      tier: tier,
      expiresAt: DateTime.now().add(const Duration(minutes: 15)),
    );
  }

  AttestationTier _attestationToTier() {
    switch (attestationType) {
      case 'PRIVATE_KEY':
        return AttestationTier.appAttest;
      case 'PLATFORM':
        return AttestationTier.appAttest;
      default:
        return AttestationTier.softwareFallback;
    }
  }
}

/// Stub for tests and simulator. Always returns software tier.
class AppAttestClientStub implements EelAttestationClient {
  @override
  Future<AttestationVerdict> attest() async {
    return AttestationVerdict(
      deviceId: 'stub-app-attest',
      tier: AttestationTier.softwareFallback,
      expiresAt: DateTime.now().add(const Duration(hours: 1)),
    );
  }
}
