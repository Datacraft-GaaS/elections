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

import 'package:eel_diaspora_kit/src/pouch_screen.dart';
import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

void main() {
  Future<void> pump(WidgetTester tester) => tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: PouchScreen()),
        ),
      );

  // The screen rendered two fixed cards reading "Ballot Box (1) • Verified •
  // Sealed" and "Form 34A (2) • Verified • Sealed" under a banner stating the
  // contents "are sealed and verified by BVI back-end". None of it came from
  // anywhere, and PouchNotifier has no loader, so no pouch is ever loaded.
  testWidgets('never claims a pouch is verified', (tester) async {
    await pump(tester);
    expect(find.byType(EelResultCard), findsNothing);
    expect(find.textContaining('Verified • Sealed'), findsNothing);
    expect(find.textContaining('sealed and verified by BVI'), findsNothing);
  });

  testWidgets('says no pouch is loaded and nothing is verified',
      (tester) async {
    await pump(tester);
    expect(find.textContaining('No pouch is loaded'), findsOneWidget);
    expect(find.textContaining('nothing here is verified'), findsOneWidget);
  });

  testWidgets('the action explains why it cannot verify', (tester) async {
    await pump(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Verify Pouch'));
    await tester.pump();
    expect(
      find.textContaining('needs the BVI back-end'),
      findsOneWidget,
    );
  });
}
