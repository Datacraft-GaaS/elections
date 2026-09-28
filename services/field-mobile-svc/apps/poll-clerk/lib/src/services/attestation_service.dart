// Real attestation via kit-sdk (ADR-0021: hard-fail MDM attestation before any auth).
// Replaces always-pass stub with EelAttestationClient (real Play Integrity / App Attest).
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

class AttestationService {
  final EelAttestationClient _client;

  AttestationService({EelAttestationClient? client})
      : _client = client ?? EelAttestationClient();

  /// Attest device via platform channel + backend verdict.
  /// Throws [AttestationFailure] if any tier fails (hardware/MEETS_DEVICE_INTEGRITY/basic/integrity verdict).
  Future<AttestationResult> attest() async {
    final verdict = await _client.verify();
    if (verdict == AttestationVerdict.meetsAll) {
      return const AttestationResult(passed: true);
    }
    return AttestationResult(
      passed: false,
      failureReason: 'Attestation verdict: ${verdict.name}',
    );
  }
}

class AttestationResult {
  final bool passed;
  final String? token;
  final String? failureReason;

  const AttestationResult({
    required this.passed,
    this.token,
    this.failureReason,
  });
}
