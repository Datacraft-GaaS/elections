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
