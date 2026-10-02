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
