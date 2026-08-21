import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';
import 'package:eel_legacy_android/src/legacy_home.dart';

void main() {
  testWidgets('LegacyHomeScreen renders with shim banner', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: LegacyHomeScreen(shimEnabled: true),
      ),
    );
    expect(find.text('eel Legacy Kit'), findsOneWidget);
    expect(find.textContaining('Legacy Android shim active'), findsOneWidget);
    expect(find.text('BVR Enrolment'), findsOneWidget);
    expect(find.text('BVI Identification'), findsOneWidget);
  });

  testWidgets('LegacyHomeScreen renders without shim banner when disabled', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: LegacyHomeScreen(shimEnabled: false),
      ),
    );
    expect(find.textContaining('Legacy Android shim active'), findsNothing);
  });
}
