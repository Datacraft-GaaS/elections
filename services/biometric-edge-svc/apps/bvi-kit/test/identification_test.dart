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

import 'package:eel_bvi_kit/src/time_fence.dart';
import 'package:eel_bvi_kit/src/gnss_guard.dart';

void main() {
  group('TimeFence', () {
    test('session allowed when skew is zero', () {
      final fence = TimeFence(now: () => DateTime(2026, 8, 21, 9, 0, 0));
      fence.recordNtsTime(DateTime(2026, 8, 21, 9, 0, 0));
      expect(fence.sessionAllowed, isTrue);
    });

    test('session allowed when skew is exactly 5 min', () {
      final nts = DateTime(2026, 8, 21, 9, 0, 0);
      final fence = TimeFence(
          now: () => nts.add(const Duration(seconds: maxSkewSeconds)));
      fence.recordNtsTime(nts);
      expect(fence.sessionAllowed, isTrue);
    });

    test('session refused when skew exceeds 5 min', () {
      final nts = DateTime(2026, 8, 21, 9, 0, 0);
      final fence = TimeFence(
          now: () => nts.add(const Duration(seconds: maxSkewSeconds + 1)));
      fence.recordNtsTime(nts);
      expect(fence.sessionAllowed, isFalse);
      expect(fence.check(), TimeFenceState.exceeded);
    });

    test('state is unknown before NTS sync', () {
      final fence = TimeFence();
      expect(fence.check(), TimeFenceState.unknown);
      expect(fence.sessionAllowed, isFalse);
    });
  });

  group('GnssGuard', () {
    test('consistent constellations pass', () {
      final guard = GnssGuard();
      final readings = [
        const GnssReading(lat: -1.292, lon: 36.821, constellation: 'GPS', fixQuality: 2),
        const GnssReading(lat: -1.292, lon: 36.821, constellation: 'GLONASS', fixQuality: 2),
      ];
      expect(guard.checkConstellations(readings), GnssGuardResult.pass);
    });

    test('large delta between constellations triggers spoof', () {
      final guard = GnssGuard();
      final readings = [
        const GnssReading(lat: -1.292, lon: 36.821, constellation: 'GPS', fixQuality: 2),
        const GnssReading(lat: -2.000, lon: 37.000, constellation: 'GLONASS', fixQuality: 2),
      ];
      expect(guard.checkConstellations(readings), GnssGuardResult.spoof);
    });

    test('single reading is undetermined', () {
      final guard = GnssGuard();
      expect(
        guard.checkConstellations([
          const GnssReading(lat: -1.292, lon: 36.821, constellation: 'GPS', fixQuality: 2),
        ]),
        GnssGuardResult.undetermined,
      );
    });

    test('imu within envelope passes', () {
      final guard = GnssGuard();
      expect(
        guard.checkImu(const ImuReading(accelMagnitudeMps2: 1.0, gyroRadPerS: 0.01)),
        GnssGuardResult.pass,
      );
    });

    test('high acceleration triggers spoof', () {
      final guard = GnssGuard();
      expect(
        guard.checkImu(const ImuReading(accelMagnitudeMps2: 9.9, gyroRadPerS: 0.5)),
        GnssGuardResult.spoof,
      );
    });
  });
}
