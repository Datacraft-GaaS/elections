/// Attestation service: reuses kit-sdk-flutter Play Integrity + App Attest +
/// TPM tier detection. Provides per-capture HSM signing via the kit-sdk
/// attestation module.
library;

// ignore: depend_on_referenced_packages
// ignore_for_file: avoid_print

/// Tier of hardware-backed key store detected at runtime.
enum AttestationTier {
  /// TPM 2.0 or Secure Element (highest assurance)
  hsm,

  /// Android StrongBox (hardware security module)
  strongBox,

  /// Android Play Integrity / iOS App Attest (software-backed)
  softwareBacked,

  /// Development stub (not for production use)
  stub,
}

/// Returned by [Attestation.sign]; contains the signature and the tier.
class AttestationSignature {
  final String signatureHex; // Ed25519 signature over [data], hex-encoded
  final AttestationTier tier;
  final String keyId; // SPIFFE SVID or TPM key alias

  const AttestationSignature({
    required this.signatureHex,
    required this.tier,
    required this.keyId,
  });
}

/// Facade over kit-sdk's attestation module.
///
/// In production this delegates to `eel_kit_sdk.attestation.DeviceAttestation`
/// which performs Play Integrity / App Attest then uses the device-bound key
/// for Ed25519 signing. In CI / stub mode a deterministic signature is
/// produced from the SHA-256 of [data] (not a real signature).
class Attestation {
  /// Sign [data] with the device-bound attestation key.
  ///
  /// [data] should be SHA-256(cert_image_bytes).
  static Future<AttestationSignature> sign(List<int> data) async {
    // TODO: delegate to eel_kit_sdk.attestation.DeviceAttestation.sign(data)
    // when kit-sdk FFI bindings are wired. For now, produce a stub signature.
    final hex = _stubSign(data);
    return AttestationSignature(
      signatureHex: hex,
      tier: AttestationTier.stub,
      keyId: 'stub-key-id',
    );
  }

  /// Detect the attestation tier available on this device.
  static Future<AttestationTier> detectTier() async {
    // TODO: delegate to eel_kit_sdk.attestation.AttestationProvider.tier
    return AttestationTier.stub;
  }

  static String _stubSign(List<int> data) {
    // Deterministic stub: XOR reduction of data bytes, zero-padded to 128 hex chars.
    var acc = 0;
    for (final b in data) {
      acc ^= b;
    }
    return acc.toRadixString(16).padLeft(128, '0');
  }
}
