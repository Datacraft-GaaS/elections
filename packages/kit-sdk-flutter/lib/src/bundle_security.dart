/// Signed-bundle cryptography for eel kits.
///
/// Covers three primitives shared by kit applications:
///
/// 1. Canonical JSON ([eelCanonicalJson]) used as the signature input for
///    configuration and mode bundles.
/// 2. ECDSA P-256 verification of signed bundles fetched from the bvi-config
///    endpoint ([EelSignedBundle]). The trusted public key is provisioned
///    into [EelSecureStore] during kit bootstrap.
/// 3. The Ed25519 device signing key ([EelSigningKey]) used to sign forms and
///    captures. The software fallback persists its seed in [EelSecureStore];
///    TEE-backed implementations (Strongbox / Secure Enclave) are wired at
///    kit runtime behind the same interface.
library bundle_security;

import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';
import 'package:pointycastle/export.dart' as pc;

import 'secure_store.dart';

final _ed25519 = Ed25519();

const int _coordinateBytes = 32;
const int _signatureBytes = 2 * _coordinateBytes;

BigInt _fromBytes(List<int> bytes) {
  var value = BigInt.zero;
  for (final byte in bytes) {
    value = (value << 8) | BigInt.from(byte);
  }
  return value;
}

Uint8List _toFixedBytes(BigInt value, int length) {
  final out = Uint8List(length);
  var v = value;
  for (var i = length - 1; i >= 0; i--) {
    out[i] = (v & BigInt.from(0xff)).toInt();
    v = v >> 8;
  }
  if (v.sign > 0) {
    throw ArgumentError.value(value, 'value', 'does not fit in $length bytes');
  }
  return out;
}

pc.SecureRandom _seededRandom() {
  final seedSource = Random.secure();
  final seed = Uint8List.fromList(
    List<int>.generate(32, (_) => seedSource.nextInt(256)),
  );
  return pc.FortunaRandom()..seed(pc.KeyParameter(seed));
}

/// Encode [value] as canonical JSON.
///
/// Object keys are sorted lexicographically, whitespace between tokens is
/// omitted, and strings use standard JSON escaping, so equal maps always
/// produce byte-identical output regardless of insertion order.
String eelCanonicalJson(Object? value) {
  final out = StringBuffer();
  _writeCanonicalJson(value, out);
  return out.toString();
}

void _writeCanonicalJson(Object? value, StringBuffer out) {
  if (value is Map) {
    final keys = value.keys.map((k) => k.toString()).toList()..sort();
    out.write('{');
    var first = true;
    for (final key in keys) {
      if (!first) out.write(',');
      first = false;
      out.write(json.encode(key));
      out.write(':');
      _writeCanonicalJson(value[key], out);
    }
    out.write('}');
  } else if (value is List) {
    out.write('[');
    for (var i = 0; i < value.length; i++) {
      if (i > 0) out.write(',');
      _writeCanonicalJson(value[i], out);
    }
    out.write(']');
  } else if (value is String) {
    out.write(json.encode(value));
  } else if (value is bool) {
    out.write(value ? 'true' : 'false');
  } else if (value is num) {
    out.write(value.toString());
  } else if (value == null) {
    out.write('null');
  } else {
    throw ArgumentError.value(
      value,
      'value',
      'unsupported canonical JSON type ${value.runtimeType}',
    );
  }
}

/// ECDSA P-256 signature helpers for signed configuration bundles.
abstract final class EelSignedBundle {
  /// Generate a fresh P-256 key pair. Used by provisioning tooling and by
  /// tests that produce signed fixtures.
  static pc.AsymmetricKeyPair<pc.PublicKey, pc.PrivateKey> generateKeyPair() {
    final generator = pc.ECKeyGenerator()
      ..init(
        pc.ParametersWithRandom(
          pc.ECKeyGeneratorParameters(pc.ECDomainParameters('prime256v1')),
          _seededRandom(),
        ),
      );
    return generator.generateKeyPair();
  }

  /// Encode a P-256 public key in the bundle wire format: base64 of the SEC1
  /// uncompressed point encoding (0x04 || X || Y).
  static String encodePublicKey(pc.ECPublicKey publicKey) {
    final point = publicKey.Q;
    if (point == null) {
      throw ArgumentError.value(publicKey, 'publicKey', 'point at infinity');
    }
    return base64Encode(point.getEncoded(false));
  }

  /// Sign [payload] (which must not yet contain [signatureKey]) with
  /// [privateKey] and return the completed bundle including the signature
  /// field: base64 of r (32 bytes) || s (32 bytes), big-endian.
  static Map<String, Object?> sign(
    Map<String, Object?> payload, {
    required pc.ECPrivateKey privateKey,
    String signatureKey = 'signature',
  }) {
    if (payload.containsKey(signatureKey)) {
      throw ArgumentError.value(payload, 'payload', 'already signed');
    }
    final message = utf8.encode(eelCanonicalJson(payload));
    final signer = pc.ECDSASigner(pc.SHA256Digest())
      ..init(
        true,
        pc.ParametersWithRandom(
          pc.PrivateKeyParameter<pc.ECPrivateKey>(privateKey),
          _seededRandom(),
        ),
      );
    final signature = signer.generateSignature(message) as pc.ECSignature;
    return {
      ...payload,
      signatureKey: base64Encode([
        ..._toFixedBytes(signature.r, _coordinateBytes),
        ..._toFixedBytes(signature.s, _coordinateBytes),
      ]),
    };
  }

  /// Verify the [signatureKey] field of [bundle] as an ECDSA P-256 signature
  /// over the canonical JSON of the bundle with the signature field removed.
  ///
  /// Returns false (never throws) when the signature or public key is
  /// malformed, or when the signature does not validate against
  /// [publicKeyBase64].
  static bool verify(
    Map<String, Object?> bundle, {
    required String publicKeyBase64,
    String signatureKey = 'signature',
  }) {
    final sigField = bundle[signatureKey];
    if (sigField is! String || sigField.isEmpty) return false;
    try {
      final sigBytes = base64Decode(sigField);
      if (sigBytes.length != _signatureBytes) return false;
      final curve = pc.ECDomainParameters('prime256v1');
      final point = curve.curve.decodePoint(base64Decode(publicKeyBase64));
      if (point == null) return false;
      final signature = pc.ECSignature(
        _fromBytes(sigBytes.sublist(0, _coordinateBytes)),
        _fromBytes(sigBytes.sublist(_coordinateBytes)),
      );
      final verifier = pc.ECDSASigner(pc.SHA256Digest())
        ..init(
          false,
          pc.PublicKeyParameter<pc.ECPublicKey>(
            pc.ECPublicKey(point, curve),
          ),
        );
      final payload = Map<String, Object?>.from(bundle)
        ..remove(signatureKey);
      return verifier.verifySignature(
        utf8.encode(eelCanonicalJson(payload)),
        signature,
      );
    } on FormatException {
      return false;
    } on ArgumentError {
      return false;
    }
  }
}

/// Secure-store slots holding the bundle trust state.
///
/// Both values are provisioned during kit bootstrap from the bvi-config
/// endpoint. Reads fail closed: callers treat an absent or unreadable value
/// as "no trust anchor" and must reject all bundles.
abstract final class ModeBundleTrustSlots {
  /// Base64 ECDSA P-256 public key trusted to sign mode/config bundles.
  static const trustedPublicKey = 'eel.mode_bundle.trusted_pubkey_v1';

  /// Canonical JSON of the most recent verified bundle applied on this kit.
  static const appliedBundle = 'eel.mode_bundle.applied_v1';
}

/// Ed25519 device signing key.
///
/// [loadOrCreate] loads the persisted seed from secure storage or creates a
/// new one on first use. When secure storage is unavailable (for example in
/// widget tests or before platform plugin registration) an ephemeral key is
/// used instead: signatures stay valid and verifiable via [publicKeyBase64]
/// but do not survive process restarts.
final class EelSigningKey {
  EelSigningKey._(
    this._keyPair,
    this.publicKeyBase64, {
    required this.persisted,
  });

  static const _seedSlot = 'eel.kit.signing_seed_v1';
  static EelSigningKey? _cached;

  final SimpleKeyPair _keyPair;

  /// Base64 Ed25519 public key matching signatures produced by [sign].
  final String publicKeyBase64;

  /// True when the seed was loaded from (or written to) secure storage.
  final bool persisted;

  /// Load the device signing key, creating it on first use. The instance is
  /// cached per process so repeated calls sign with the same key.
  static Future<EelSigningKey> loadOrCreate({EelSecureStore? store}) async {
    final existing = _cached;
    if (existing != null) return existing;
    final storage = store ?? EelSecureStore();

    SimpleKeyPair keyPair;
    var persisted = true;
    // Only the secure-storage round trip is guarded: any failure there
    // (plugin missing, binding uninitialized, platform error) degrades to an
    // ephemeral key rather than failing the whole kit.
    try {
      final seedB64 = await storage.read(_seedSlot);
      if (seedB64 != null) {
        keyPair = await _ed25519.newKeyPairFromSeed(base64Decode(seedB64));
      } else {
        keyPair = await _ed25519.newKeyPair();
        final seed = await keyPair.extractPrivateKeyBytes();
        await storage.write(_seedSlot, base64Encode(seed));
      }
    } catch (_) {
      persisted = false;
      keyPair = await _ed25519.newKeyPair();
    }

    final pub = await keyPair.extractPublicKey();
    return _cached = EelSigningKey._(
      keyPair,
      base64Encode(pub.bytes),
      persisted: persisted,
    );
  }

  /// Sign [message] and return the base64 Ed25519 signature.
  Future<String> sign(List<int> message) async {
    final sig = await _ed25519.sign(message, keyPair: _keyPair);
    return base64Encode(sig.bytes);
  }
}
