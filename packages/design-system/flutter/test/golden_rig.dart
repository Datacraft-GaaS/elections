import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:eel_design/eel_design.dart';

/// Shared pump helper for goldens + widget tests.
/// [settle] false for screens with infinite animations (spinners).
Future<void> pumpEel(
  WidgetTester tester,
  Widget child, {
  Brightness brightness = Brightness.light,
  Size size = const Size(390, 844),
  bool settle = true,
}) async {
  await tester.binding.setSurfaceSize(size);
  addTearDown(() => tester.binding.setSurfaceSize(null));
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    MaterialApp(
      theme: brightness == Brightness.dark ? EelTheme.dark() : EelTheme.light(),
      home: Scaffold(body: child),
    ),
  );
  if (settle) {
    await tester.pumpAndSettle();
  } else {
    await tester.pump(const Duration(milliseconds: 300));
  }
}

/// Standard golden wrapper: labelled device frame on a theme background.
Widget goldenWrap(String label, Widget child, {Brightness? brightness}) =>
    child;
