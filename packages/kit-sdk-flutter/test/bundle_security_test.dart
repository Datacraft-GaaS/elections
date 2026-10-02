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

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:pointycastle/export.dart' as pc;

import 'package:eel_kit_sdk/eel_kit_sdk.dart';

void main() {
  group('eelCanonicalJson', () {
    test('is independent of key insertion order', () {
      final a = eelCanonicalJson({'b': 1, 'a': {'z': true, 'y': 'x'}});
      final b = eelCanonicalJson({'a': {'y': 'x', 'z': true}, 'b': 1});
      expect(a, b);
    });

    test('produces compact sorted output', () {
      expect(
        eelCanonicalJson({'b': '2', 'a': [1, null]}),
        '{"a":[1,null],"b":"2"}',
      );
    });

    test('escapes strings as JSON', () {
      expect(eelCanonicalJson({'k': 'he"llo'}), '{"k":"he\\"llo"}');
    });

    test('rejects non-JSON types', () {
      expect(() => eelCanonicalJson(Object()), throwsArgumentError);
    });
  });

  group('EelSignedBundle', () {
    pc.ECPublicKey publicKeyOf(pc.AsymmetricKeyPair<pc.PublicKey, pc.PrivateKey> pair) =>
        pair.publicKey as pc.ECPublicKey;

    pc.ECPrivateKey privateKeyOf(pc.AsymmetricKeyPair<pc.PublicKey, pc.PrivateKey> pair) =>
        pair.privateKey as pc.ECPrivateKey;

    String encoded(pc.ECPublicKey pub) => EelSignedBundle.encodePublicKey(pub);

    test('sign and verify round-trip', () {
      final pair = EelSignedBundle.generateKeyPair();
      final bundle = EelSignedBundle.sign(
        {
          'election_type': 'referendum',
          'valid_until': '2030-01-01T00:00:00Z',
          'bundle_id': 'ref-2026',
          'valid_from': '2026-01-01T00:00:00Z',
        },
        privateKey: privateKeyOf(pair),
      );
      expect(
        EelSignedBundle.verify(bundle, publicKeyBase64: encoded(publicKeyOf(pair))),
        isTrue,
      );
    });

    test('tampered payload fails verification', () {
      final pair = EelSignedBundle.generateKeyPair();
      var bundle = EelSignedBundle.sign(
        {'bundle_id': 'b1'},
        privateKey: privateKeyOf(pair),
      );
      bundle = {...bundle, 'bundle_id': 'evil'};
      expect(
        EelSignedBundle.verify(bundle, publicKeyBase64: encoded(publicKeyOf(pair))),
        isFalse,
      );
    });

    test('wrong key fails verification', () {
      final signerPair = EelSignedBundle.generateKeyPair();
      final otherPair = EelSignedBundle.generateKeyPair();
      final bundle = EelSignedBundle.sign(
        {'a': 1},
        privateKey: privateKeyOf(signerPair),
      );
      expect(
        EelSignedBundle.verify(bundle, publicKeyBase64: encoded(publicKeyOf(otherPair))),
        isFalse,
      );
    });

    test('malformed signature or key returns false instead of throwing', () {
      final otherPair = EelSignedBundle.generateKeyPair();
      final otherPubB64 = encoded(publicKeyOf(otherPair));
      for (final badSig in ['', 'not-base64!!', 'AAAA']) {
        expect(
          EelSignedBundle.verify(
            {'signature': badSig},
            publicKeyBase64: otherPubB64,
          ),
          isFalse,
        );
      }
      final goodBundle = EelSignedBundle.sign(
        {'a': 1},
        privateKey: privateKeyOf(otherPair),
      );
      for (final badKey in ['', 'not-base64!!', 'AAAA']) {
        expect(
          EelSignedBundle.verify(goodBundle, publicKeyBase64: badKey),
          isFalse,
        );
      }
    });
  });

  group('EelSigningKey', () {
    test('signatures are deterministic per message', () async {
      final key = await EelSigningKey.loadOrCreate();
      final a = await key.sign(utf8.encode('form-payload'));
      final b = await key.sign(utf8.encode('form-payload'));
      expect(a, b);
    });

    test('different messages produce different signatures', () async {
      final key = await EelSigningKey.loadOrCreate();
      final a = await key.sign(utf8.encode('one'));
      final b = await key.sign(utf8.encode('two'));
      expect(a, isNot(b));
    });
  });
}
