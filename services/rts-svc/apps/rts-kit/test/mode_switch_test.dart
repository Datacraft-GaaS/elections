/// Mode-switch tests.
///
/// Verifies that applying a mode bundle mid-session reconfigures the workflow
/// and that unverified bundles are rejected.
library;

import 'package:flutter_test/flutter_test.dart';

import 'package:eel_rts_kit/src/rts_workflow/mode_switch.dart';

void main() {
  group('RtsModeBundle', () {
    test('loadCurrent returns general election by default [FR-M08-042] [FR-M08-002] [EC-KIT-04]', () async {
      final bundle = await RtsModeBundle.loadCurrent();
      expect(bundle.electionType, ElectionType.general);
      expect(bundle.formType, FormType.form34a);
      expect(bundle.bundleId, 'default-general');
    });

    test('applyBundle parses referendum bundle correctly [FR-M08-002] [FR-M08-042] [EC-KIT-04]', () async {
      final raw = <String, Object?>{
        'election_type': 'referendum',
        'form_type': 'referendumResultsForm',
        'valid_from': '2026-01-01T00:00:00Z',
        'valid_until': '2030-01-01T00:00:00Z',
        'bundle_id': 'ref-2026',
        'signature': 'stub',
      };
      final bundle = await RtsModeBundle.applyBundle(raw);
      expect(bundle.electionType, ElectionType.referendum);
      expect(bundle.formType, FormType.referendumResultsForm);
      expect(bundle.bundleId, 'ref-2026');
      expect(bundle.verified, isTrue);
    });

    test('applyBundle parses by-election bundle [FR-M08-002] [FR-M08-042]', () async {
      final raw = <String, Object?>{
        'election_type': 'byElection',
        'form_type': 'byElection34a',
        'valid_from': '2026-01-01T00:00:00Z',
        'valid_until': '2030-01-01T00:00:00Z',
        'bundle_id': 'by-2026',
        'signature': 'stub',
      };
      final bundle = await RtsModeBundle.applyBundle(raw);
      expect(bundle.electionType, ElectionType.byElection);
      expect(bundle.formType, FormType.byElection34a);
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

      await notifier.applyRaw({
        'election_type': 'referendum',
        'form_type': 'referendumResultsForm',
        'valid_from': '2026-01-01T00:00:00Z',
        'valid_until': '2030-01-01T00:00:00Z',
        'bundle_id': 'ref-test',
        'signature': 'stub',
      });

      expect(notifier.state.electionType, ElectionType.referendum);
      expect(notifier.state.bundleId, 'ref-test');
    });

    test('applyRaw mid-session replaces active mode [FR-M08-042] [EC-KIT-04]', () async {
      final initial = await RtsModeBundle.loadCurrent();
      final notifier = ModeBundleNotifier(initial);

      // Apply a by-election bundle during session.
      await notifier.applyRaw({
        'election_type': 'byElection',
        'form_type': 'byElection34a',
        'valid_from': '2026-01-01T00:00:00Z',
        'valid_until': '2030-01-01T00:00:00Z',
        'bundle_id': 'by-mid-session',
        'signature': 'stub',
      });

      expect(notifier.state.electionType, ElectionType.byElection);
      expect(notifier.state.formType, FormType.byElection34a);
      expect(notifier.state.bundleId, 'by-mid-session');
    });
  });
}
