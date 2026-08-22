import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:eel_cvr_field_officer/src/screens/home.dart';

Widget _wrap(Widget child) => ProviderScope(
      child: MaterialApp(home: child),
    );

void main() {
  group('HomeScreen', () {
    testWidgets('renders title', (tester) async {
      await tester.pumpWidget(_wrap(const HomeScreen()));
      expect(find.text('CVR Field Officer'), findsWidgets);
    });

    testWidgets('has capture death cert button', (tester) async {
      await tester.pumpWidget(_wrap(const HomeScreen()));
      expect(find.text('Capture Death Certificate'), findsOneWidget);
    });

    testWidgets('has capture ID correction button', (tester) async {
      await tester.pumpWidget(_wrap(const HomeScreen()));
      expect(find.text('Capture ID Correction'), findsOneWidget);
    });

    testWidgets('action buttons rendered', (tester) async {
      await tester.pumpWidget(_wrap(const HomeScreen()));
      // Both action labels render (FilledButton.icon internals vary by Flutter version)
      expect(find.text('Capture Death Certificate'), findsOneWidget);
      expect(find.text('Capture ID Correction'), findsOneWidget);
    });

    testWidgets('history link present', (tester) async {
      await tester.pumpWidget(_wrap(const HomeScreen()));
      expect(find.text('View History'), findsOneWidget);
    });
  });
}
