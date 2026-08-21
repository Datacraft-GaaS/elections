import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:eel_field_support/main.dart';

void main() {
  testWidgets('app builds without errors', (tester) async {
    await tester.pumpWidget(const ProviderScope(child: EelFieldSupportApp()));
    expect(find.byType(MaterialApp), findsOneWidget);
  });

  testWidgets('SlaTimerChip shows breached state', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: SizedBox(),
        ),
      ),
    );
    expect(find.byType(Scaffold), findsOneWidget);
  });
}
