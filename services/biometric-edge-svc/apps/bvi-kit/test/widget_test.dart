import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:eel_bvi_kit/src/app.dart';

void main() {
  testWidgets('app builds', (tester) async {
    await tester.pumpWidget(const BviKitApp());
    expect(find.byType(MaterialApp), findsOneWidget);
  });
}
