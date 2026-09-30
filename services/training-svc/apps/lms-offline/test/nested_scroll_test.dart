import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'package:eel_lms_offline/features/modules/module_screen.dart';
import 'package:eel_lms_offline/features/assessment/assessment_screen.dart';

/// These screens used a ListView as the body of EelScaffold's scrollable
/// variant, which is a scrollable inside a scrollable. That throws during
/// layout rather than at analysis, so each is pumped to prove it lays out.
void main() {
  testWidgets('ModuleScreen lays out', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: MaterialApp(home: ModuleScreen())),
    );
    await tester.pump();
    expect(tester.takeException(), isNull);
    expect(find.byType(Scaffold), findsOneWidget);
  });
  testWidgets('AssessmentScreen lays out', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: MaterialApp(home: AssessmentScreen())),
    );
    await tester.pump();
    expect(tester.takeException(), isNull);
    expect(find.byType(Scaffold), findsOneWidget);
  });
}
