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
import 'package:eel_poll_clerk/src/features/voter_lookup/voter_lookup_screen.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

void main() {
  Future<void> pump(WidgetTester tester) => tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: VoterLookupScreen()),
        ),
      );

  // The screen used to render a fixed "Voter Verified — Station KE-NBI-001 •
  // Registered" card above a dead Confirm Lookup button, so a poll clerk saw a
  // verified voter who had never been looked up.
  testWidgets('never claims a voter is verified', (tester) async {
    await pump(tester);
    expect(find.text('Voter Verified'), findsNothing);
    expect(find.textContaining('Registered'), findsNothing);
    expect(find.byType(EelResultCard), findsNothing);
  });

  testWidgets('says verification is unavailable', (tester) async {
    await pump(tester);
    expect(
      find.textContaining('not yet wired to the register'),
      findsOneWidget,
    );
  });

  testWidgets('the action says why nothing happens', (tester) async {
    await pump(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Confirm Lookup'));
    await tester.pump();
    expect(find.textContaining('register lookup backend is not implemented'),
        findsOneWidget);
  });
}
