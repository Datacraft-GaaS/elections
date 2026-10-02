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
  const sampling = AdaptiveSampling();

  group('AdaptiveSampling', () {
    test('100 Hz during active workflow with sufficient battery', () {
      final mode = sampling.mode(batteryPercent: 80, workflowActive: true);
      expect(mode, equals(SamplingMode.activeWorkflow));
      expect(sampling.hzFor(mode), equals(100));
    });

    test('10 Hz when idle with sufficient battery', () {
      final mode = sampling.mode(batteryPercent: 80, workflowActive: false);
      expect(mode, equals(SamplingMode.idle));
      expect(sampling.hzFor(mode), equals(10));
    });

    test('1 Hz when battery below 15 %', () {
      final mode = sampling.mode(batteryPercent: 14, workflowActive: true);
      expect(mode, equals(SamplingMode.lowBattery));
      expect(sampling.hzFor(mode), equals(1));
    });

    test('1 Hz exactly at 15 %', () {
      final mode = sampling.mode(batteryPercent: 15, workflowActive: true);
      expect(mode, equals(SamplingMode.lowBattery));
    });

    test('10 Hz when idle at 16 %', () {
      final mode = sampling.mode(batteryPercent: 16, workflowActive: false);
      expect(mode, equals(SamplingMode.idle));
    });

    test('interval for activeWorkflow is 10 ms (100 Hz)', () {
      expect(
        sampling.intervalFor(SamplingMode.activeWorkflow),
        equals(const Duration(milliseconds: 10)),
      );
    });

    test('interval for idle is 100 ms (10 Hz)', () {
      expect(
        sampling.intervalFor(SamplingMode.idle),
        equals(const Duration(milliseconds: 100)),
      );
    });

    test('interval for lowBattery is 1 s (1 Hz)', () {
      expect(
        sampling.intervalFor(SamplingMode.lowBattery),
        equals(const Duration(seconds: 1)),
      );
    });
  });
}
