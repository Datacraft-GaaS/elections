import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:eel_design/eel_design.dart';

import 'golden_rig.dart';

// Golden files for every core widget × light/dark. Regenerate:
// flutter test --update-goldens
void main() {
  final widgets = <String, WidgetBuilder>{
    'eel_button_primary': (_) => EelButton(
          label: 'Submit',
          onPressed: () {},
        ),
    'eel_button_loading': (_) => const EelButton(
          label: 'Submit',
          onPressed: null,
          loading: true,
        ),
    'eel_button_destructive': (_) => EelButton(
          label: 'Delete',
          onPressed: () {},
          variant: EelButtonVariant.destructive,
        ),
    'eel_button_ghost': (_) => EelButton(
          label: 'Cancel',
          onPressed: () {},
          variant: EelButtonVariant.ghost,
        ),
    'eel_text_field': (_) => const EelTextField(
          label: 'Station',
          hint: 'ST-001',
          helper: 'From your duty roster',
        ),
    'eel_text_field_error': (_) => const EelTextField(
          label: 'Station',
          errorText: 'Station not recognised',
        ),
    'eel_banner_info': (_) => const EelBanner(
          title: 'Offline',
          message: 'Work continues offline.',
        ),
    'eel_banner_danger': (_) => const EelBanner(
          title: 'Network blocked',
          message: 'The server could not be verified.',
          severity: EelSeverity.danger,
        ),
    'eel_status_chips': (_) => const Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            EelStatusChip(label: 'Synced', severity: EelSeverity.success),
            SizedBox(height: 8),
            EelStatusChip(label: 'Rate limited', severity: EelSeverity.warning),
            SizedBox(height: 8),
            EelStatusChip(label: 'Failed', severity: EelSeverity.danger),
          ],
        ),
    'eel_empty_state': (_) => const EelEmptyState(
          title: 'No incidents',
          message: 'Reported incidents will appear here.',
        ),
    'eel_error_state': (_) => const EelErrorState(
          problem: EelProblem(
            type: 'https://eel.dev/errors/conflict',
            title: 'Duplicate record',
            status: 422,
            detail: 'A record with this idempotency key exists.',
            instance: '/incidents/it-42',
          ),
        ),
    'eel_offline_state': (_) => const EelOfflineState(
          title: 'You are offline',
          message: '3 items waiting to sync.',
        ),
    'eel_result_card': (_) => const EelResultCard(
          titleText: 'Form 34A',
          proofStatus: 'signature valid',
          rows: [
            (label: 'Station', value: 'ST-001'),
            (label: 'Seal', value: 'SL-99231'),
          ],
        ),
    'eel_seal_stamp': (_) =>
        const EelSealStamp(sealId: 'SL-99231', verified: true),
    'eel_sync_indicator': (_) => const Padding(
          padding: EdgeInsets.all(16),
          child: EelSyncIndicator(pendingCount: 3, lastSync: null),
        ),
    'eel_step_flow': (_) => EelStepFlow(
          steps: const ['Scan', 'Verify', 'Submit'],
          currentStep: 1,
          onStepTapped: (_) {},
          onNext: () {},
          child: const Center(child: Text('Step content')),
        ),
    'eel_select': (_) => const EelSelect<String>(
          label: 'Region',
          items: [
            DropdownMenuItem(value: 'nrb', child: Text('Nairobi')),
          ],
          value: 'nrb',
          onChanged: null,
        ),
    'eel_search_field': (_) => const Padding(
          padding: EdgeInsets.all(16),
          child: EelSearchField(),
        ),
    'eel_pin_field': (_) => Padding(
          padding: const EdgeInsets.all(16),
          child: EelPinField(length: 6, onCompleted: (v) {}),
        ),
    'eel_min_hit_target': (_) => const Center(
          child: EelMinHitTarget(
            onTap: null,
            child: Icon(Icons.camera_alt),
          ),
        ),
    'eel_connectivity_banners': (_) => const Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            EelConnectivityBanner(state: EelConnectivityState.offline),
            SizedBox(height: 8),
            EelConnectivityBanner(state: EelConnectivityState.degraded),
          ],
        ),
  };

  testWidgets('core widgets light + dark', (tester) async {
    for (final entry in widgets.entries) {
      await pumpEel(tester, Center(child: Builder(builder: entry.value)),
          settle: false);
      await tester.pump(const Duration(milliseconds: 400));
      await expectLater(
        find.byType(MaterialApp),
        matchesGoldenFile('goldens/${entry.key}.light.png'),
      );
      await pumpEel(
        tester,
        Center(child: Builder(builder: entry.value)),
        brightness: Brightness.dark,
        settle: false,
      );
      await tester.pump(const Duration(milliseconds: 400));
      await expectLater(
        find.byType(MaterialApp),
        matchesGoldenFile('goldens/${entry.key}.dark.png'),
      );
    }
  });

  testWidgets('text scales to 200% without clipping [a11y]', (tester) async {
    // Overflow check: key text widgets at 2.0 scale must not throw layout
    // overflow errors. Full golden coverage per app (plan §5.2).
    tester.view.devicePixelRatio = 1.0;
    tester.view.physicalSize = const Size(390, 844);
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      MediaQuery(
        data: const MediaQueryData(textScaler: TextScaler.linear(2.0)),
        child: MaterialApp(
          theme: EelTheme.light(),
          home: Scaffold(
            body: SingleChildScrollView(
              child: Column(
                children: [
                  const EelTextField(label: 'Station', hint: 'ST-001'),
                  EelButton(label: 'Submit', onPressed: () {}),
                  const EelBanner(
                      title: 'Offline', message: 'Work continues offline.'),
                  const EelStatusChip(
                      label: 'Synced', severity: EelSeverity.success),
                ],
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pump();
    expect(tester.takeException(), isNull);
  });
}
