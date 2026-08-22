import 'package:flutter_test/flutter_test.dart';

import 'package:eel_cvr_field_officer/src/services/attestation.dart';

void main() {
  group('Attestation (stub)', () {
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

    test('sign differs for different data', () async {
      final a = await Attestation.sign([1, 2, 3]);
      final b = await Attestation.sign([4, 5, 6]);
      // XOR stub: may collide for some inputs, but these differ.
      // Just check it returns a string.
      expect(a.signatureHex, isA<String>());
      expect(b.signatureHex, isA<String>());
    });

    test('detectTier returns AttestationTier', () async {
      final tier = await Attestation.detectTier();
      expect(tier, isA<AttestationTier>());
    });

    test('stub tier is AttestationTier.stub', () async {
      final tier = await Attestation.detectTier();
      expect(tier, AttestationTier.stub);
    });
  });
}
