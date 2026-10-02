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
