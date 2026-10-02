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
  group('DropDetector', () {
    test('3g spike triggers DropSignal within 200 ms', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      int counter = 0;

      final detector = DropDetector(
        deviceId: 'dev-001',
        sink: queue,
        nextTraceId: () => 'trace',
        nextSpanId: () => 'span',
        nextCounter: () => counter++,
        currentPrevHash: () => chain.tail,
      );

      final stopwatch = Stopwatch()..start();

      // Feed a 5g spike (9.81 * 5 m/s²).
      await detector.onAccelerometer(0, 0, 9.81 * 5);

      stopwatch.stop();

      expect(queue.pendingCount, greaterThan(0));
      expect(stopwatch.elapsedMilliseconds, lessThan(200));

      final batch = queue.drainBatch();
      final json = batch.first.signalJson;
      expect(json['signalType'], equals('drop'));
      expect((json['peakAccelG'] as num).toDouble(), greaterThanOrEqualTo(3.0));
    });

    test('sub-3g movement does not trigger DropSignal', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      int counter = 0;

      final detector = DropDetector(
        deviceId: 'dev-001',
        sink: queue,
        nextTraceId: () => 'trace',
        nextSpanId: () => 'span',
        nextCounter: () => counter++,
        currentPrevHash: () => chain.tail,
      );

      // Feed a 2g spike — below the 3g threshold.
      await detector.onAccelerometer(0, 0, 9.81 * 2);

      expect(queue.pendingCount, equals(0));
    });

    test('free-fall followed by spike is classified as free-fall drop', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      int counter = 0;

      final detector = DropDetector(
        deviceId: 'dev-001',
        sink: queue,
        nextTraceId: () => 'trace',
        nextSpanId: () => 'span',
        nextCounter: () => counter++,
        currentPrevHash: () => chain.tail,
      );

      // Free-fall: near-zero G for several samples.
      for (var i = 0; i < 5; i++) {
        await detector.onAccelerometer(0.1, 0.1, 0.1);
      }
      // High-G impact.
      await detector.onAccelerometer(0, 0, 9.81 * 4);

      expect(queue.pendingCount, greaterThan(0));
      final batch = queue.drainBatch();
      final json = batch.first.signalJson;
      expect(json['freeFallDetected'], isTrue);
    });
  });
}
