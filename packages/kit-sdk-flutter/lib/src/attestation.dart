import 'package:flutter/foundation.dart';

enum AttestationTier { playIntegrity, appAttest, tpm, softwareFallback }

class AttestationVerdict {
  const AttestationVerdict({
    required this.deviceId,
    required this.tier,
    required this.expiresAt,
  });

  final String deviceId;
  final AttestationTier tier;
  final DateTime expiresAt;
}

abstract class EelAttestationClient {
  Future<AttestationVerdict> attest();
}

/// Software-tier stub for development and testing. Real backends (Play Integrity,
/// App Attest, TPM) are wired at kit runtime.
class EelAttestationClientStub implements EelAttestationClient {
  @override
  Future<AttestationVerdict> attest() async {
    return AttestationVerdict(
      deviceId: 'stub-device-${defaultTargetPlatform.name}',
      tier: AttestationTier.softwareFallback,
      expiresAt: DateTime.now().add(const Duration(hours: 1)),
    );
  }
}
