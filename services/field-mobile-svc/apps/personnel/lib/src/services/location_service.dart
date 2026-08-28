import 'package:flutter/foundation.dart';
import 'package:geolocator/geolocator.dart';

/// A single GPS reading used for attendance and spoof checking.
@immutable
class GpsFix {
  final double lat;
  final double lon;
  final double accuracyM;

  const GpsFix({
    required this.lat,
    required this.lon,
    required this.accuracyM,
  });

  /// A reading is suspect when accuracy is worse than 100 m — the same
  /// threshold the dispatch-api spoof detector applies (design §M33).
  bool get isSpoofed => accuracyM > 100.0;

  /// Local clock time as `HH:mm`, used on sealed confirmations.
  String get clockLabel {
    final now = DateTime.now();
    return '${now.hour.toString().padLeft(2, '0')}:${now.minute.toString().padLeft(2, '0')}';
  }
}

/// Source of the device location.
abstract class LocationProvider {
  Future<GpsFix> currentFix();
}

class LocationException implements Exception {
  final String message;
  const LocationException(this.message);

  @override
  String toString() => message;
}

/// Deterministic, clearly-marked simulated fix (Nairobi CBD). Used in tests
/// and demos; the field kit uses [DeviceLocationProvider].
class SimulatedLocationProvider implements LocationProvider {
  const SimulatedLocationProvider();

  @override
  Future<GpsFix> currentFix() async {
    await Future<void>.delayed(const Duration(milliseconds: 500));
    return const GpsFix(lat: -1.2921, lon: 36.8219, accuracyM: 8.0);
  }
}

/// Real device GPS via the geolocator plugin, with permission handling. Throws
/// [LocationException] with a user-facing reason when the radio is off or
/// permission is denied.
class DeviceLocationProvider implements LocationProvider {
  const DeviceLocationProvider();

  @override
  Future<GpsFix> currentFix() async {
    final enabled = await Geolocator.isLocationServiceEnabled();
    if (!enabled) {
      throw const LocationException('Location services are off. Turn on location and try again.');
    }

    var permission = await Geolocator.checkPermission();
    if (permission == LocationPermission.denied) {
      permission = await Geolocator.requestPermission();
      if (permission == LocationPermission.denied) {
        throw const LocationException('Location permission denied.');
      }
    }
    if (permission == LocationPermission.deniedForever) {
      throw const LocationException('Location permission is permanently denied. Enable it in Settings.');
    }

    final pos = await Geolocator.getCurrentPosition(
      locationSettings: const LocationSettings(
        accuracy: LocationAccuracy.best,
        timeLimit: Duration(seconds: 30),
      ),
    );

    return GpsFix(lat: pos.latitude, lon: pos.longitude, accuracyM: pos.accuracy);
  }
}
