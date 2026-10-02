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

//! These screens must not report figures nobody measured.
//!
//! `QueueScreen` rendered `Total in queue: 5`, `Avg wait: 4 min` and
//! `Position 3 of 5 • Turnout: 45%`. `KioskScreen` rendered
//! `Operational - 100% uptime` and `Total voters: 12,450 • Turnout: 87% • EOD: 18:00`.
//! `EODReportScreen` rendered the same literals twice.
//!
//! Every one was a `const`, so nothing was ever measured and the analyzer had no
//! reason to complain. An election official reading "Turnout: 87%" has no way to
//! tell a measured value from a typed one — and a turnout figure is a count of
//! people who have voted, not an estimate.

import 'package:eel_design/eel_design.dart';
import 'package:eel_poll_clerk/src/features/eod_report/eod_report_screen.dart';
import 'package:eel_poll_clerk/src/features/kiosk_mode/kiosk_screen.dart';
import 'package:eel_poll_clerk/src/features/queue/queue_screen.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

/// Every string the widget tree renders.
Future<List<String>> renderedText(WidgetTester tester, Widget screen) async {
  await tester.pumpWidget(
    ProviderScope(child: MaterialApp(home: Scaffold(body: screen))),
  );
  await tester.pumpAndSettle();
  return tester
      .widgetList<Text>(find.byType(Text))
      .map((t) => t.data ?? '')
      .toList();
}

/// A figure that looks measured but is not.
///
/// The label must be followed by a **digit**. A bare `uptime` or `queue:` would
/// match the honest disclaimers too — "so uptime is not measured" is the right
/// thing to render, and this pattern must not flag it.
final _fabricatedFigure = RegExp(
  r'Turnout:\s*\d|Total voters:\s*\d|[Qq]ueue:\s*\d|\d+\s*%\s*uptime|'
  r'Battery:\s*\d|Incidents open:\s*\d',
);

void main() {
  testWidgets('QueueScreen states the queue is unavailable', (tester) async {
    final text = await renderedText(tester, const QueueScreen());
    for (final line in text) {
      expect(line, isNot(matches(_fabricatedFigure)), reason: 'QueueScreen rendered a figure: $line');
    }
    expect(
      text.any((t) => t.contains('Unavailable') || t.contains('unavailable')),
      isTrue,
      reason: 'QueueScreen must say the data is unavailable: $text',
    );
  });

  testWidgets('KioskScreen reports no turnout and no uptime', (tester) async {
    final text = await renderedText(tester, const KioskScreen());
    for (final line in text) {
      expect(line, isNot(matches(_fabricatedFigure)), reason: 'KioskScreen rendered a figure: $line');
    }
  });

  testWidgets('EODReportScreen reports no turnout and no uptime', (tester) async {
    final text = await renderedText(tester, const EODReportScreen());
    for (final line in text) {
      expect(line, isNot(matches(_fabricatedFigure)), reason: 'EODReportScreen rendered a figure: $line');
    }
  });

  testWidgets('a control that cannot act is visibly disabled', (tester) async {
    // The buttons were `onPressed: () {}`, which renders a live control that does
    // nothing when tapped. `EelButton` treats a null callback as disabled.
    await tester.pumpWidget(
      const ProviderScope(child: MaterialApp(home: QueueScreen())),
    );
    await tester.pumpAndSettle();
    // `EelButton` is not a ButtonStyleButton, so assert on the widget itself.
    final buttons = tester.widgetList<EelButton>(find.byType(EelButton));
    expect(buttons, isNotEmpty, reason: 'the screen should still render its control');
    for (final b in buttons) {
      expect(
        b.onPressed,
        isNull,
        reason: 'a control with no wired action must render disabled, not live',
      );
    }
  });
}
