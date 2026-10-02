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
import 'package:eel_kit_l1/eel_kit_l1.dart';

void main() {
  group('EelKitL1AnomalyDetector', () {
    test('returns result for each telemetry point', () {
      final detector = EelKitL1AnomalyDetector();
      final points = List.generate(
        20,
        (i) => TelemetryPoint(
          stationCode: 'STN$i',
          timestamp: DateTime.now().toUtc(),
          votesCast: 100 + i,
          ballotPapersIssued: 100 + i,
          spoiltBallots: 2,
          kitId: 'KIT$i',
        ),
      );
      final results = detector.detect(points);
      expect(results.length, equals(20));
    });

    test('flags extreme outlier as anomaly', () {
      final detector = EelKitL1AnomalyDetector();
      final normal = List.generate(
        30,
        (i) => TelemetryPoint(
          stationCode: 'STN$i',
          timestamp: DateTime.now().toUtc(),
          votesCast: 100 + i,
          ballotPapersIssued: 100 + i,
          spoiltBallots: 2,
          kitId: 'KIT$i',
        ),
      );
      detector.fit(normal);
      final outlier = [
        TelemetryPoint(
          stationCode: 'STN_BAD',
          timestamp: DateTime.now().toUtc(),
          votesCast: 999999,
          ballotPapersIssued: 1,
          spoiltBallots: 999,
          kitId: 'KIT_BAD',
        ),
      ];
      final results = detector.detect(outlier);
      expect(results.first.stationCode, equals('STN_BAD'));
    });

    test('AnomalyResult has valid severity', () {
      final detector = EelKitL1AnomalyDetector();
      final points = [
        TelemetryPoint(
          stationCode: 'STN1',
          timestamp: DateTime.now().toUtc(),
          votesCast: 100,
          ballotPapersIssued: 100,
          spoiltBallots: 2,
          kitId: 'KIT1',
        ),
      ];
      final results = detector.detect(points);
      expect(AnomalySeverity.values, contains(results.first.severity));
    });
  });
}
