/// Mode-switch tests.
///
/// Verifies that applying a mode bundle mid-session reconfigures the workflow
/// and that unverified bundles are rejected (fail closed).
library;

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:pointycastle/export.dart' as pc;

import 'package:eel_rts_kit/src/rts_workflow/mode_switch.dart';

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
      'form_type': 'referendumResultsForm',
      'valid_from': '2026-01-01T00:00:00Z',
      'valid_until': '2030-01-01T00:00:00Z',
      'bundle_id': 'ref-2026',
    };

void main() {
  group('RtsModeBundle', () {
    test('loadCurrent returns general election by default [FR-M08-042] [FR-M08-002] [EC-KIT-04]', () async {
      final bundle = await RtsModeBundle.loadCurrent();
      expect(bundle.electionType, ElectionType.general);
      expect(bundle.formType, FormType.form34a);
      expect(bundle.bundleId, 'default-general');
    });

    test('applyBundle verifies a properly signed referendum bundle [FR-M08-002] [FR-M08-042] [EC-KIT-04]', () async {
      final (raw, anchor) = _signedBundle(_referendumPayload());
      final bundle = await RtsModeBundle.applyBundle(raw, trustAnchorB64: anchor);
      expect(bundle.electionType, ElectionType.referendum);
      expect(bundle.formType, FormType.referendumResultsForm);
      expect(bundle.bundleId, 'ref-2026');
      expect(bundle.verified, isTrue);
    });

    test('applyBundle parses by-election bundle [FR-M08-002] [FR-M08-042]', () async {
      final (raw, anchor) = _signedBundle({
        'election_type': 'byElection',
        'form_type': 'byElection34a',
        'valid_from': '2026-01-01T00:00:00Z',
        'valid_until': '2030-01-01T00:00:00Z',
        'bundle_id': 'by-2026',
      });
      final bundle = await RtsModeBundle.applyBundle(raw, trustAnchorB64: anchor);
      expect(bundle.electionType, ElectionType.byElection);
      expect(bundle.formType, FormType.byElection34a);
      expect(bundle.verified, isTrue);
    });

    test('applyBundle rejects a tampered bundle (signature no longer valid) [EC-KIT-04]', () async {
      final (raw, anchor) = _signedBundle(_referendumPayload());
      final tampered = Map<String, Object?>.from(raw)
        ..['election_type'] = 'general';
      final bundle =
          await RtsModeBundle.applyBundle(tampered, trustAnchorB64: anchor);
      expect(bundle.verified, isFalse);
    });

    test('applyBundle fails closed when no trust anchor is provisioned [EC-KIT-04]', () async {
      final (raw, _) = _signedBundle(_referendumPayload());
      final bundle = await RtsModeBundle.applyBundle(raw);
      expect(bundle.verified, isFalse);
    });

    test('isActive returns false for expired bundle [FR-M08-042]', () {
      final expired = RtsModeBundle(
        electionType: ElectionType.general,
        formType: FormType.form34a,
        validFrom: DateTime.utc(2020, 1, 1),
        validUntil: DateTime.utc(2021, 1, 1),
        bundleId: 'old',
        verified: true,
      );
      expect(expired.isActive, isFalse);
    });

    test('isActive returns true for current bundle [FR-M08-042]', () {
      final active = RtsModeBundle(
        electionType: ElectionType.general,
        formType: FormType.form34a,
        validFrom: DateTime.utc(2020, 1, 1),
        validUntil: DateTime.utc(2099, 1, 1),
        bundleId: 'active',
        verified: true,
      );
      expect(active.isActive, isTrue);
    });
  });

  group('ModeBundleNotifier', () {
    test('applyRaw updates state and reflects in provider [FR-M08-042] [EC-KIT-04]', () async {
      final initial = await RtsModeBundle.loadCurrent();
      final notifier = ModeBundleNotifier(initial);

      expect(notifier.state.electionType, ElectionType.general);

      final (raw, anchor) = _signedBundle({
        ..._referendumPayload(),
        'bundle_id': 'ref-test',
      },);
      await notifier.applyRaw(raw, trustAnchorB64: anchor);

      expect(notifier.state.electionType, ElectionType.referendum);
      expect(notifier.state.bundleId, 'ref-test');
    });

    test('applyRaw mid-session replaces active mode [FR-M08-042] [EC-KIT-04]', () async {
      final initial = await RtsModeBundle.loadCurrent();
      final notifier = ModeBundleNotifier(initial);

      // Apply a by-election bundle during session.
      final (raw, anchor) = _signedBundle({
        'election_type': 'byElection',
        'form_type': 'byElection34a',
        'valid_from': '2026-01-01T00:00:00Z',
        'valid_until': '2030-01-01T00:00:00Z',
        'bundle_id': 'by-mid-session',
      });
      await notifier.applyRaw(raw, trustAnchorB64: anchor);

      expect(notifier.state.electionType, ElectionType.byElection);
      expect(notifier.state.formType, FormType.byElection34a);
      expect(notifier.state.bundleId, 'by-mid-session');
    });
  });
}
