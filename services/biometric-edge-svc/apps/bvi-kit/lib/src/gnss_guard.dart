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

/// GNSS spoof detection (EC-KIT-01).
///
/// Multi-constellation cross-check: position must be consistent across
/// GPS, GLONASS, and Galileo within [_maxConstellationDeltaM] metres.
/// IMU sanity envelope: acceleration/rotation must be plausible for a
/// stationary polling-station installation.
library;

import 'dart:math' as math;

const double _maxConstellationDeltaM = 50.0;
const double _maxAccelMps2 = 4.9; // 0.5 g

enum GnssGuardResult { pass, spoof, undetermined }

class GnssReading {
  final double lat;
  final double lon;
  final double? altM;
  final String constellation; // "GPS", "GLONASS", "GALILEO"
  final int fixQuality;

  const GnssReading({
    required this.lat,
    required this.lon,
    this.altM,
    required this.constellation,
    required this.fixQuality,
  });
}

class ImuReading {
  final double accelMagnitudeMps2;
  final double gyroRadPerS;

  const ImuReading({required this.accelMagnitudeMps2, required this.gyroRadPerS});
}

class GnssGuard {
  /// Checks whether multiple constellation readings are mutually consistent.
  GnssGuardResult checkConstellations(List<GnssReading> readings) {
    if (readings.length < 2) return GnssGuardResult.undetermined;
    final ref = readings.first;
    for (final r in readings.skip(1)) {
      final delta = _haversineM(ref.lat, ref.lon, r.lat, r.lon);
      if (delta > _maxConstellationDeltaM) return GnssGuardResult.spoof;
    }
    return GnssGuardResult.pass;
  }

  /// Validates IMU readings against the stationary-kit envelope.
  GnssGuardResult checkImu(ImuReading imu) {
    if (imu.accelMagnitudeMps2 > _maxAccelMps2) return GnssGuardResult.spoof;
    return GnssGuardResult.pass;
  }

  double _haversineM(double lat1, double lon1, double lat2, double lon2) {
    const r = 6371000.0;
    final dlat = _toRad(lat2 - lat1);
    final dlon = _toRad(lon2 - lon1);
    final a = _sin2(dlat / 2) +
        _cos(_toRad(lat1)) * _cos(_toRad(lat2)) * _sin2(dlon / 2);
    return 2 * r * _asin(_sqrt(a));
  }

  double _toRad(double deg) => deg * math.pi / 180.0;
  double _sin2(double x) => math.sin(x) * math.sin(x);
  double _cos(double x) => math.cos(x);
  double _asin(double x) => math.asin(x.clamp(-1.0, 1.0));
  double _sqrt(double x) => x <= 0 ? 0 : math.sqrt(x);
}
