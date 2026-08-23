/// Attestation service: reuses kit-sdk-flutter Play Integrity + App Attest +
/// TPM tier detection. Provides per-capture signing via the kit-sdk
/// attestation module.
library;

import 'dart:convert';
// ignore_for_file: avoid_print

import 'package:eel_kit_sdk/eel_kit_sdk.dart' as kit;

/// Tier of hardware-backed key store detected at runtime.
enum AttestationTier {
  /// TPM 2.0 or Secure Element (highest assurance)
  hsm,

  /// Android StrongBox (hardware security module)
  strongBox,

  /// Android Play Integrity / iOS App Attest (software-backed)
  softwareBacked,

  /// Development fallback: no hardware-backed attestation available
  softwareFallback,
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
/// Tier detection delegates to `eel_kit_sdk`'s attestation client, which
/// performs Play Integrity / App Attest against the backend and reports the
/// device tier.
/// Signing note: [Attestation.sign] signs with the device-bound Ed25519 key
/// from eel_kit_sdk's [kit.EelSigningKey]. The seed persists in
/// EelSecureStore; on devices with a TEE (Strongbox / Secure Enclave) the
/// platform-backed implementation replaces the software fallback behind the
/// same interface.
class Attestation {
  static final kit.EelAttestationClient _client = kit.EelAttestationClientStub();

  static kit.EelSigningKey? _signingKey;

  /// Sign [data] with the device-bound attestation key.
  ///
  /// [data] should be SHA-256(cert_image_bytes).
  static Future<AttestationSignature> sign(List<int> data) async {
    final verdict = await _client.attest();
    final key = _signingKey ??= await kit.EelSigningKey.loadOrCreate();
    final sigB64 = await key.sign(data);
    return AttestationSignature(
      signatureHex:
          base64Decode(sigB64).map((b) => b.toRadixString(16).padLeft(2, '0')).join(),
      tier: _mapTier(verdict.tier),
      keyId: 'eel-kit-${verdict.deviceId}',
    );
  }

  /// Detect the attestation tier available on this device.
  static Future<AttestationTier> detectTier() async {
    final verdict = await _client.attest();
    return _mapTier(verdict.tier);
  }

  /// Map kit-sdk tiers onto this app's tier scale.
  ///
  /// Play Integrity / App Attest verify app integrity but hold no key, so
  /// they map to softwareBacked; the SDK's software fallback maps to
  /// softwareFallback.
  static AttestationTier _mapTier(kit.AttestationTier tier) =>
      switch (tier) {
        kit.AttestationTier.tpm => AttestationTier.hsm,
        kit.AttestationTier.playIntegrity ||
        kit.AttestationTier.appAttest =>
          AttestationTier.softwareBacked,
        kit.AttestationTier.softwareFallback => AttestationTier.softwareFallback,
      };
}
