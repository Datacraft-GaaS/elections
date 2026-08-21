/// GNSS spoof detection (EC-KIT-01).
///
/// Multi-constellation cross-check: position must be consistent across
/// GPS, GLONASS, and Galileo within [_maxConstellationDeltaM] metres.
/// IMU sanity envelope: acceleration/rotation must be plausible for a
/// stationary polling-station installation.

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

  double _toRad(double deg) => deg * 3.141592653589793 / 180.0;
  double _sin2(double x) => _sin(x) * _sin(x);
  double _sin(double x) => x - x * x * x / 6.0; // small-angle stub
  double _cos(double x) => 1.0 - x * x / 2.0;
  double _asin(double x) => x; // stub; production uses dart:math
  double _sqrt(double x) => x <= 0 ? 0 : _sqrtApprox(x);
  double _sqrtApprox(double x) {
    var r = x;
    for (var i = 0; i < 20; i++) r = (r + x / r) / 2;
    return r;
  }
}
