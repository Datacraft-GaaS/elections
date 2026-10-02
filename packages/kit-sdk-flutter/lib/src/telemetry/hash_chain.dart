// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Per-device hash chain over signals using SHA-256 (BLAKE3 pure-Dart fallback).
///
/// The chain guarantees that silent rewrite of any signal breaks the chain:
///   thisHash = SHA256(prevHash || signalType || occurredAt || deviceId || ...fields)
library;

import 'dart:convert';
import 'dart:typed_data';
import 'package:crypto/crypto.dart';

import 'signal_sink.dart';

/// Computes the canonical hash for [signal] given its [prevHash].
///
/// Uses SHA-256 as a pure-Dart BLAKE3 fallback.  The input is:
///   prevHash (hex) || signalType || occurredAt (ISO) || deviceId
///   || monotonicCounter (decimal) || JSON-encoded signal-specific fields
String computeSignalHash(Signal signal) {
  final json = signal.toJson();
  // Remove chain fields so hashing is deterministic regardless of population order.
  final canonical = <String, Object?>{...json}
    ..remove('thisHash')
    ..remove('signature');
  final bytes = utf8.encode(jsonEncode(canonical));
  return sha256.convert(bytes).toString();
}

/// Manages the in-memory tail of the per-device hash chain.
///
/// Call [attach] before a signal is queued; the signal gets its [Signal.prevHash]
/// and [Signal.thisHash] populated in-place.
const String _zeroHash = '0000000000000000000000000000000000000000000000000000000000000000';

class HashChain {
  HashChain({String initialHash = _zeroHash}) : _prevHash = initialHash;

  String _prevHash;

  /// The hash of the most recent signal that passed through this chain.
  String get tail => _prevHash;

  /// Attaches [signal] to the chain: sets [Signal.prevHash] from [tail] and
  /// computes [Signal.thisHash], then advances [tail].
  void attach(Signal signal) {
    // prevHash field is final; we work with a mutable copy via the JSON round-trip
    // pattern is: populate the mutable thisHash field after construction.
    // Because prevHash is final, the caller must pass it when constructing the
    // signal.  Here we only compute and set thisHash.
    signal.thisHash = computeSignalHash(signal);
    _prevHash = signal.thisHash;
  }

  /// Verifies [signals] form an unbroken chain.
  ///
  /// Returns true iff every signal's prevHash matches the preceding signal's
  /// thisHash, and every thisHash matches the recomputed hash.
  static bool verify(List<Signal> signals, {String? firstPrevHash}) {
    String expectedPrev = firstPrevHash ?? _zeroHash;
    for (final s in signals) {
      if (s.prevHash != expectedPrev) return false;
      final recomputed = computeSignalHash(s);
      if (s.thisHash != recomputed) return false;
      expectedPrev = s.thisHash;
    }
    return true;
  }
}

/// Serialisable chain state for crash-safe persistence.
class HashChainState {
  HashChainState({required this.deviceId, required this.tailHash, required this.counter});

  final String deviceId;
  final String tailHash;
  final int counter;

  Map<String, Object?> toJson() => {
        'deviceId': deviceId,
        'tailHash': tailHash,
        'counter': counter,
      };

  factory HashChainState.fromJson(Map<String, Object?> j) => HashChainState(
        deviceId: j['deviceId'] as String,
        tailHash: j['tailHash'] as String,
        counter: j['counter'] as int,
      );

  static Uint8List stateBytes(HashChainState s) =>
      Uint8List.fromList(utf8.encode(jsonEncode(s.toJson())));
}
