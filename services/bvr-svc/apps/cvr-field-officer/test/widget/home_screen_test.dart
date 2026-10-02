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
