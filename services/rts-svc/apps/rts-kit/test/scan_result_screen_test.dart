import 'package:eel_design/eel_design.dart';
import 'package:eel_rts_kit/src/rts_workflow/scan_result_screen.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

void main() {
  Future<void> pump(WidgetTester tester) => tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: ScanResultScreen()),
        ),
      );

  // This screen rendered a fixed card reading "Scan Verified" naming tag
  // RT-001 at station KE-NBI-001. Neither was read from anything, so a tally
  // or transfer could be confirmed against a scan that never happened.
  testWidgets('does not claim a scan was verified', (tester) async {
    await pump(tester);
    expect(find.text('Scan Verified'), findsNothing);
    expect(find.textContaining('RT-001'), findsNothing);
    expect(find.textContaining('KE-NBI-001'), findsNothing);
  });

  testWidgets('says no scan was recorded', (tester) async {
    await pump(tester);
    expect(find.text('No scan recorded'), findsOneWidget);
  });

  testWidgets('the action explains there is nothing to confirm',
      (tester) async {
    await pump(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Confirm Scan'));
    await tester.pump();
    expect(find.textContaining('no scan to confirm'), findsOneWidget);
  });
}
