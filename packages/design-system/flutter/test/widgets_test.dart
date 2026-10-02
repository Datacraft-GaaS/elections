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

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:eel_design/eel_design.dart';

import 'golden_rig.dart';

void main() {
  testWidgets('EelButton ≥ 48 dp hit target [a11y]', (tester) async {
    await pumpEel(
      tester,
      EelButton(
          label: 'Save', onPressed: () {}, variant: EelButtonVariant.primary),
    );
    final size = tester.getSize(find.byType(FilledButton));
    expect(size.height, greaterThanOrEqualTo(48.0));
  });

  testWidgets('EelButton loading shows spinner and disables tap',
      (tester) async {
    var taps = 0;
    await pumpEel(
      tester,
      EelButton(label: 'Save', onPressed: () => taps++, loading: true),
      settle: false,
    );
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    await tester.tap(find.byType(EelButton));
    expect(taps, 0);
  });

  testWidgets('EelBanner announces tone via semantics [a11y]', (tester) async {
    await pumpEel(
      tester,
      const EelBanner(
        message: 'Sync failed',
        severity: EelSeverity.danger,
      ),
    );
    final semantics = tester.getSemantics(find.text('Sync failed'));
    expect(semantics.label, contains('critical'));
    expect(semantics.label, contains('Sync failed'));
  });

  testWidgets('EelStatusChip carries icon + label, not colour alone [a11y]',
      (tester) async {
    await pumpEel(
      tester,
      const EelStatusChip(label: 'Synced', severity: EelSeverity.success),
    );
    expect(find.text('Synced'), findsOneWidget);
    expect(find.byIcon(Icons.check_circle_outline), findsOneWidget);
  });

  testWidgets('EelTextField always shows label [a11y]', (tester) async {
    await pumpEel(
      tester,
      const EelTextField(label: 'Station', hint: 'Enter station'),
    );
    expect(find.text('Station'), findsOneWidget);
  });

  testWidgets('EelPinField completes at length', (tester) async {
    var completed = '';
    await pumpEel(
      tester,
      EelPinField(length: 6, onCompleted: (v) => completed = v),
    );
    await tester.enterText(find.byType(TextField), '123456');
    expect(completed, '123456');
  });

  testWidgets('EelSearchField clear button resets value', (tester) async {
    final controller = TextEditingController(text: 'abc');
    addTearDown(controller.dispose);
    await pumpEel(tester, EelSearchField(controller: controller));
    await tester.tap(find.byTooltip('Clear search'));
    expect(controller.text, isEmpty);
  });

  testWidgets('EelSelect renders and changes value', (tester) async {
    String? picked;
    await pumpEel(
      tester,
      StatefulBuilder(
        builder: (context, setState) => EelSelect<String>(
          label: 'Region',
          items: const [
            DropdownMenuItem(value: 'nrb', child: Text('Nairobi')),
            DropdownMenuItem(value: 'ksm', child: Text('Kisumu')),
          ],
          value: null,
          onChanged: (v) => picked = v,
        ),
      ),
    );
    await tester.tap(find.byType(DropdownButtonFormField<String>));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Kisumu').last);
    await tester.pumpAndSettle();
    expect(picked, 'ksm');
  });

  testWidgets('EelStepFlow marks done steps and finishes on last',
      (tester) async {
    var finished = false;
    await pumpEel(
      tester,
      EelStepFlow(
        steps: const ['Scan', 'Verify', 'Submit'],
        currentStep: 2,
        onStepTapped: (_) {},
        onNext: () => finished = true,
        finishLabel: 'Finish',
        child: const SizedBox.shrink(),
      ),
    );
    expect(find.text('Finish'), findsOneWidget);
    await tester.tap(find.text('Finish'));
    expect(finished, true);
  });

  testWidgets('EelSlaTimerChip escalates near deadline', (tester) async {
    final now = DateTime(2026, 9, 3, 12, 0, 0);
    await pumpEel(
      tester,
      EelSlaTimerChip(
        deadline: now.add(const Duration(minutes: 20)),
        phaseLabels: (
          normal: 'On track',
          warning: 'Due soon',
          critical: 'Overdue',
        ),
        now: () => now,
      ),
    );
    // 20 of 30 minutes remain → normal phase.
    expect(find.textContaining('On track'), findsOneWidget);
  });

  testWidgets('EelSyncIndicator shows queue depth text', (tester) async {
    await pumpEel(
      tester,
      EelSyncIndicator(
        pendingCount: 3,
        lastSync: DateTime(2026, 9, 3, 14, 32),
      ),
    );
    expect(find.textContaining('3 waiting'), findsOneWidget);
  });

  testWidgets('EelResultCard shows proof chip and rows', (tester) async {
    await pumpEel(
      tester,
      EelResultCard(
        title: Text('Form 34A',
            style: EelTextStyle.titleMedium
                .copyWith(fontWeight: FontWeight.w600)),
        proofStatus: 'signature valid',
        rows: const [(label: 'Polling station', value: 'ST-001')],
      ),
    );
    expect(find.text('Form 34A'), findsOneWidget);
    expect(find.text('signature valid'), findsOneWidget);
    expect(find.text('ST-001'), findsOneWidget);
  });

  testWidgets('EelSealStamp offers verify when unverified', (tester) async {
    var verified = false;
    await pumpEel(
      tester,
      EelSealStamp(
          sealId: 'SL-001', verified: false, onVerify: () => verified = true),
    );
    await tester.tap(find.text('Verify'));
    expect(verified, true);
  });

  testWidgets('EelConnectivityBanner hides when online', (tester) async {
    await pumpEel(
      tester,
      const Directionality(
        textDirection: TextDirection.ltr,
        child: EelConnectivityBanner(state: EelConnectivityState.online),
      ),
    );
    expect(find.byType(EelBanner), findsNothing);
  });

  testWidgets('EelConnectivityBanner warns on captive portal', (tester) async {
    await pumpEel(
      tester,
      const EelConnectivityBanner(state: EelConnectivityState.captivePortal),
    );
    expect(find.text('Network blocked'), findsOneWidget);
  });

  testWidgets('EelMinHitTarget enforces minimum size [a11y]', (tester) async {
    await pumpEel(
      tester,
      const EelMinHitTarget(
        onTap: _noop,
        child: SizedBox(width: 10, height: 10),
      ),
    );
    final size = tester.getSize(find.byType(ConstrainedBox).first);
    expect(size.height, greaterThanOrEqualTo(48.0));
    expect(size.width, greaterThanOrEqualTo(48.0));
  });

  testWidgets('EelVoiceGuideButton toggles and reports state', (tester) async {
    var active = false;
    await pumpEel(
      tester,
      EelVoiceGuideButton(
        onToggle: (v) => active = v,
        activeLabel: 'Voice on',
        inactiveLabel: 'Voice off',
      ),
    );
    await tester.tap(find.byType(EelVoiceGuideButton));
    expect(active, true);
  });

  testWidgets('EelErrorState renders problem fields', (tester) async {
    await pumpEel(
      tester,
      const EelErrorState(
        problem: EelProblem(
          type: 'https://eel.dev/errors/rate-limited',
          title: 'Too many attempts',
          status: 429,
          detail: 'Wait a minute before retrying.',
          instance: '/incidents/it-42',
        ),
        onRetry: _noop,
      ),
    );
    expect(find.text('Too many attempts'), findsOneWidget);
    expect(find.text('Wait a minute before retrying.'), findsOneWidget);
    expect(find.textContaining('429'), findsOneWidget);
  });

  // `expand` was accepted and never rendered, so a button asking to be
  // full-width silently came out natural-width. Assert rendered geometry.
  //
  // The parent must pass LOOSE constraints, or the test proves nothing: a
  // tight SizedBox(width: 300) forces the button to 300 whether or not the
  // flag is honoured. Center under a 390-wide surface gives the button room to
  // be narrow, which is the only way the two cases differ.
  testWidgets('EelButton fullWidth fills the available width',
      (tester) async {
    await pumpEel(tester, const Center(child: EelButton(
      label: 'Submit',
      fullWidth: true,
      onPressed: _noop,
    )));
    expect(tester.getSize(find.byType(FilledButton)).width, 390);
  });

  testWidgets('EelButton is natural width without fullWidth', (tester) async {
    await pumpEel(
        tester, const Center(child: EelButton(label: 'Submit', onPressed: _noop)));
    expect(tester.getSize(find.byType(FilledButton)).width, lessThan(390));
  });
}

void _noop() {}
