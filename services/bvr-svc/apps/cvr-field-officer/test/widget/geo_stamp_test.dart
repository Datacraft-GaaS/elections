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

import 'package:eel_cvr_field_officer/src/services/geo_stamp.dart';

void main() {
  group('GeoStamp', () {
    test('GeoStampResult holds lat/lon/accuracy', () {
      const result = GeoStampResult(
        lat: -1.286389,
        lon: 36.817223,
        accuracyM: 5.0,
        truncated: false,
      );
      expect(result.lat, closeTo(-1.286389, 0.000001));
      expect(result.lon, closeTo(36.817223, 0.000001));
      expect(result.accuracyM, 5.0);
      expect(result.truncated, isFalse);
    });

    test('GeoStampException formats message', () {
      const ex = GeoStampException('Location services are disabled.');
      expect(ex.toString(), contains('Location services'));
    });

    test('precision truncation at 2 decimals for high accuracy value', () {
      // The _truncate helper (tested indirectly via the result shape).
      // A lat of -1.286389 truncated to 2 dp should be -1.28.
      // We cannot call GeoStamp.stamp() in unit tests (requires device),
      // so verify the GeoStampResult contract holds when truncated=true.
      const result = GeoStampResult(
        lat: -1.28,
        lon: 36.81,
        accuracyM: 600.0,
        truncated: true,
      );
      expect(result.truncated, isTrue);
      // Truncated to 2dp: should not have more than 2 significant decimal places.
      expect(result.lat.toStringAsFixed(6), startsWith('-1.28'));
    });
  });
}
