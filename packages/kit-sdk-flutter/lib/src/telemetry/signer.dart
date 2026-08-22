/// HSM-anchored HMAC signature per signal via kit-attest FFI.
///
/// In production the device HMAC key is held in the kit-attest HSM / TEE.
/// For unit tests or when the FFI is unavailable the signer falls back to a
/// software HMAC-SHA-256 keyed from secure storage.
library;

import 'dart:convert';
import 'package:crypto/crypto.dart';

import 'signal_sink.dart';

/// Signs individual signals.
abstract class Signer {
  /// Sign [signal] and populate [Signal.signature] in-place.
  Future<void> sign(Signal signal);

  /// Verify [signal.signature] against [signal.thisHash].
  Future<bool> verify(Signal signal);
}

/// Software HMAC-SHA-256 signer — used when HSM FFI is unavailable.
class HmacSigner implements Signer {
  HmacSigner({required String deviceKey})
      : _hmac = Hmac(sha256, utf8.encode(deviceKey));

  final Hmac _hmac;

  @override
  Future<void> sign(Signal signal) async {
    assert(signal.thisHash.isNotEmpty, 'Hash must be computed before signing');
    final mac = _hmac.convert(utf8.encode(signal.thisHash));
    signal.signature = mac.toString();
  }

  @override
  Future<bool> verify(Signal signal) async {
    if (signal.signature == null) return false;
    final mac = _hmac.convert(utf8.encode(signal.thisHash));
    return mac.toString() == signal.signature;
  }
}

/// Stub signer that skips signing — for integration tests without HSM.
class NoOpSigner implements Signer {
  const NoOpSigner();

  @override
  Future<void> sign(Signal signal) async {
    signal.signature = 'noop';
  }

  @override
  Future<bool> verify(Signal signal) async => true;
}
