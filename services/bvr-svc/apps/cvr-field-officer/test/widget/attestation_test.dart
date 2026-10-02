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

import 'package:flutter_test/flutter_test.dart';

import 'package:eel_cvr_field_officer/src/services/attestation.dart';

void main() {
  group('Attestation', () {
    test('sign returns non-empty hex signature', () async {
      final sig = await Attestation.sign([1, 2, 3, 4, 5]);
      expect(sig.signatureHex.isNotEmpty, isTrue);
      // Hex chars only
      expect(RegExp(r'^[0-9a-f]+$').hasMatch(sig.signatureHex), isTrue);
    });

    test('sign is deterministic for same data', () async {
      final a = await Attestation.sign([10, 20, 30]);
      final b = await Attestation.sign([10, 20, 30]);
      expect(a.signatureHex, equals(b.signatureHex));
    });

    test('sign returns a hex-encoded Ed25519 signature', () async {
      final sig = await Attestation.sign([1, 2, 3]);
      // Ed25519 signatures are 64 bytes; hex encoding doubles the length.
      expect(sig.signatureHex.length, 128);
    });

    test('detectTier returns AttestationTier', () async {
      final tier = await Attestation.detectTier();
      expect(tier, isA<AttestationTier>());
    });

    test('stub client tier maps to softwareFallback', () async {
      final tier = await Attestation.detectTier();
      expect(tier, AttestationTier.softwareFallback);
    });

    test('sign tier matches detected tier', () async {
      final sig = await Attestation.sign([9, 9, 9]);
      expect(sig.tier, await Attestation.detectTier());
      expect(sig.keyId, startsWith('eel-kit-'));
    });
  });
}
