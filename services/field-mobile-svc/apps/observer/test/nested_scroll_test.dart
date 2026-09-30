import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'package:eel_observer/src/features/mission_export/mission_export_screen.dart';

/// These screens used a ListView as the body of EelScaffold's scrollable
/// variant, which is a scrollable inside a scrollable. That throws during
/// layout rather than at analysis, so each is pumped to prove it lays out.
void main() {
  testWidgets('MissionExportScreen lays out', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: MaterialApp(home: MissionExportScreen())),
    );
    await tester.pump();
    expect(tester.takeException(), isNull);
    expect(find.byType(Scaffold), findsOneWidget);
  });
}
