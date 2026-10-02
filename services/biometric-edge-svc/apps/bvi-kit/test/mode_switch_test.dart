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

/// BVI mode-bundle tests.
///
/// Verifies signed-bundle application, fail-closed rejection, and the
/// biometric scope rules for referendum and by-election bundles.
library;

import 'package:eel_bvi_kit/src/mode_switch.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pointycastle/export.dart' as pc;

/// Sign [payload] with a freshly generated P-256 key pair and return the
/// completed bundle together with the base64 public key as trust anchor.
(Map<String, Object?>, String) _signedBundle(Map<String, Object?> payload) {
  final pair = EelSignedBundle.generateKeyPair();
  final anchor = EelSignedBundle.encodePublicKey(pair.publicKey as pc.ECPublicKey);
  final raw = EelSignedBundle.sign(
    payload,
    privateKey: pair.privateKey as pc.ECPrivateKey,
  );
  return (raw, anchor);
}

Map<String, Object?> _referendumPayload() => {
      'election_type': 'referendum',
      'biometric_scope': 'national',
      'valid_from': '2026-01-01T00:00:00Z',
      'valid_until': '2030-01-01T00:00:00Z',
      'bundle_id': 'ref-2026',
    };

void main() {
  group('BviModeBundle', () {
    test('loadCurrent returns general election default when nothing is stored', () async {
      // No bundle or trust anchor is provisioned in the test environment,
      // so loadCurrent must fall back to the documented default.
      final bundle = await BviModeBundle.loadCurrent();
      expect(bundle.electionType, BviElectionType.general);
      expect(bundle.biometricScope, 'national');
      expect(bundle.bundleId, 'default-general');
    });

    test('applyBundle verifies a properly signed bundle [EC-KIT-04]', () async {
      final (raw, anchor) = _signedBundle(_referendumPayload());
      final bundle =
          await BviModeBundle.applyBundle(raw, trustAnchorB64: anchor);
      expect(bundle.electionType, BviElectionType.referendum);
      expect(bundle.biometricScope, 'national');
      expect(bundle.bundleId, 'ref-2026');
      expect(bundle.verified, isTrue);
    });

    test('applyBundle rejects a tampered bundle (signature no longer valid) [EC-KIT-04]', () async {
      final (raw, anchor) = _signedBundle(_referendumPayload());
      final tampered = Map<String, Object?>.from(raw)
        ..['biometric_scope'] = 'constituency:001';
      expect(
        () => BviModeBundle.applyBundle(tampered, trustAnchorB64: anchor),
        throwsA(isA<AssertionError>()),
      );
    });

    test('applyBundle fails closed when no trust anchor is provisioned [EC-KIT-04]', () async {
      final (raw, _) = _signedBundle(_referendumPayload());
      expect(
        () => BviModeBundle.applyBundle(raw),
        throwsA(isA<AssertionError>()),
      );
    });

    test('isActive returns false for expired bundle', () {
      final expired = BviModeBundle(
        electionType: BviElectionType.general,
        biometricScope: 'national',
        validFrom: DateTime.utc(2020, 1, 1),
        validUntil: DateTime.utc(2021, 1, 1),
        bundleId: 'old',
        verified: true,
      );
      expect(expired.isActive, isFalse);
    });

    test('byElectionBundle scopes biometric authentication to the constituency', () {
      final bundle = BviModeBundle.byElectionBundle(
        bundleId: 'by-2026',
        constituencyId: '001',
        validFrom: DateTime.utc(2026, 1, 1),
        validUntil: DateTime.utc(2030, 1, 1),
      );
      expect(bundle.electionType, BviElectionType.byElection);
      expect(bundle.biometricScope, 'constituency:001');
    });
  });
}
