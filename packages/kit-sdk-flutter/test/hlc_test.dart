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
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

void main() {
  group('Hlc', () {
    test('parse and format round-trip', () {
      const s = '1700000000000.0.1';
      final hlc = Hlc.parse(s);
      expect(hlc.toString(), s);
    });

    test('tick advances counter within same ms', () {
      const h = Hlc(ms: 100, counter: 5, node: 1);
      final ticked = h.tick(100);
      expect(ticked.ms, 100);
      expect(ticked.counter, 6);
    });

    test('tick advances ms when wall clock is ahead', () {
      const h = Hlc(ms: 100, counter: 5, node: 1);
      final ticked = h.tick(200);
      expect(ticked.ms, 200);
      expect(ticked.counter, 0);
    });

    test('merge takes max ms', () {
      const a = Hlc(ms: 100, counter: 0, node: 1);
      const b = Hlc(ms: 200, counter: 0, node: 2);
      final merged = a.merge(b, 50);
      expect(merged.ms, 200);
    });

    test('compareTo is total order', () {
      const a = Hlc(ms: 100, counter: 0, node: 1);
      const b = Hlc(ms: 100, counter: 1, node: 1);
      expect(a.compareTo(b), lessThan(0));
    });
  });
}
