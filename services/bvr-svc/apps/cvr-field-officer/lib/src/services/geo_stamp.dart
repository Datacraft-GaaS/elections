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

/// GeoStamp: GNSS + cell + Wi-Fi location fusion with privacy-preserving
/// precision reduction when signal quality is poor.
///
/// When geo_accuracy_m > DISTRICT_PRECISION_THRESHOLD_M the returned
/// coordinates are truncated to 2 decimal places (~1.1 km precision at the
/// equator) to avoid exposing sub-district movement patterns.
library;

import 'package:geolocator/geolocator.dart';

const double _districtPrecisionThresholdM = 500.0;

class GeoStamp {
  /// Request location permission and return current position.
  ///
  /// Throws [GeoStampException] when permission is denied or service disabled.
  static Future<GeoStampResult> stamp() async {
    final serviceEnabled = await Geolocator.isLocationServiceEnabled();
    if (!serviceEnabled) {
      throw const GeoStampException('Location services are disabled.');
    }

    var permission = await Geolocator.checkPermission();
    if (permission == LocationPermission.denied) {
      permission = await Geolocator.requestPermission();
      if (permission == LocationPermission.denied) {
        throw const GeoStampException('Location permission denied.');
      }
    }
    if (permission == LocationPermission.deniedForever) {
      throw const GeoStampException('Location permission permanently denied.');
    }

    final pos = await Geolocator.getCurrentPosition(
      locationSettings: const LocationSettings(
        accuracy: LocationAccuracy.best,
        timeLimit: Duration(seconds: 30),
      ),
    );

    double lat = pos.latitude;
    double lon = pos.longitude;
    final accuracy = pos.accuracy;

    // Privacy-preserving precision reduction below district granularity.
    if (accuracy > _districtPrecisionThresholdM) {
      lat = _truncate(lat, 2);
      lon = _truncate(lon, 2);
    }

    return GeoStampResult(
      lat: lat,
      lon: lon,
      accuracyM: accuracy,
      truncated: accuracy > _districtPrecisionThresholdM,
    );
  }

  static double _truncate(double v, int decimals) {
    final factor = _pow10(decimals);
    return (v * factor).truncateToDouble() / factor;
  }

  static double _pow10(int n) {
    var r = 1.0;
    for (var i = 0; i < n; i++) {
      r *= 10;
    }
    return r;
  }
}

class GeoStampResult {
  final double lat;
  final double lon;
  final double accuracyM;
  final bool truncated; // true when precision was reduced for privacy

  const GeoStampResult({
    required this.lat,
    required this.lon,
    required this.accuracyM,
    required this.truncated,
  });
}

class GeoStampException implements Exception {
  final String message;
  const GeoStampException(this.message);

  @override
  String toString() => 'GeoStampException: $message';
}
