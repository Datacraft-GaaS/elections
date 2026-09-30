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
