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

import 'package:citizen_verify/src/features/incident/report_screen.dart';
import 'package:citizen_verify/src/features/stations/station_finder_screen.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

/// Every screen here used a ListView as the body of EelScaffold's scrollable
/// variant — a scrollable inside a scrollable. That throws during layout rather
/// than at analysis, so each is pumped to prove it lays out.
void main() {
  Future<void> expectRenders(WidgetTester tester, Widget screen) async {
    await tester.pumpWidget(
      ProviderScope(child: MaterialApp(home: screen)),
    );
    await tester.pump();
    expect(tester.takeException(), isNull);
    expect(find.byType(Scaffold), findsOneWidget);
  }

  testWidgets('incident report screen lays out', (tester) async {
    await expectRenders(tester, const IncidentReportScreen());
  });

  testWidgets('station finder screen lays out', (tester) async {
    await expectRenders(tester, const StationFinderScreen());
  });
}
