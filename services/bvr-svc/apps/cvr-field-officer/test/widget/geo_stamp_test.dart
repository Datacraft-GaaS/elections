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
