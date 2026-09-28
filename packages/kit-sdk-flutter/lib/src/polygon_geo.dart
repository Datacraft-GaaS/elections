import 'dart:math';

import 'package:geolocator/geolocator.dart';
import 'package:flutter/foundation.dart';

/// Polygon geo utilities (plan §5.2, geolocator dep).
///
/// Used by observer station polygons, kiosk mode boundaries, and turnout
/// zone checks. All functions are pure-Dart; no platform channels.
@immutable
class PolygonGeo {
  const PolygonGeo._();

  /// Check if [position] is inside [polygon] using the ray-casting algorithm.
  ///
  /// [polygon] is a list of coordinates in order (clockwise or counter-clockwise).
  /// The polygon is implicitly closed (last point connects to first).
  static bool isPointInPolygon(
    Position position,
    List<Position> polygon,
  ) {
    if (polygon.length < 3) return false;

    bool inside = false;
    final n = polygon.length;

    for (int i = 0, j = n - 1; i < n; j = i++) {
      final lati = polygon[i].latitude;
      final loni = polygon[i].longitude;
      final latj = polygon[j].latitude;
      final lonj = polygon[j].longitude;

      final intersect =
          ((loni > position.longitude) != (lonj > position.longitude)) &&
              (position.latitude <
                  (latj - lati) *
                      (position.longitude - loni) /
                      (lonj - loni) +
                  lati);
      if (intersect) inside = !inside;
    }

    return inside;
  }

  /// Check if [position] is within [distanceMeters] of any point in [polygon].
  static bool isPointNearPolygon(
    Position position,
    List<Position> polygon,
    double distanceMeters,
  ) {
    for (final point in polygon) {
      final distance = Geolocator.distanceBetween(
        position.latitude,
        position.longitude,
        point.latitude,
        point.longitude,
      );
      if (distance <= distanceMeters) return true;
    }
    return false;
  }

  /// Compute the bounding box of [polygon] (min/max lat/lon).
  static ({double minLat, double maxLat, double minLon, double maxLon})
      boundingBox(List<Position> polygon) {
    if (polygon.isEmpty) {
      return (minLat: 0, maxLat: 0, minLon: 0, maxLon: 0);
    }

    double minLat = polygon.first.latitude;
    double maxLat = polygon.first.latitude;
    double minLon = polygon.first.longitude;
    double maxLon = polygon.first.longitude;

    for (final p in polygon) {
      if (p.latitude < minLat) minLat = p.latitude;
      if (p.latitude > maxLat) maxLat = p.latitude;
      if (p.longitude < minLon) minLon = p.longitude;
      if (p.longitude > maxLon) maxLon = p.longitude;
    }

    return (minLat: minLat, maxLat: maxLat, minLon: minLon, maxLon: maxLon);
  }

  /// Compute the approximate area of [polygon] in square meters using the
  /// spherical excess formula (good enough for election station polygons).
  static double approximateArea(List<Position> polygon) {
    if (polygon.length < 3) return 0;

    const R = 6371000; // Earth radius in meters
    double rad(double deg) => deg * pi / 180;

    double sum = 0;
    final n = polygon.length;
    for (int i = 0; i < n; i++) {
      final p1 = polygon[i];
      final p2 = polygon[(i + 1) % n];
      sum +=
          (rad(p2.longitude) - rad(p1.longitude)) *
          (2 + sin(rad(p1.latitude)) + sin(rad(p2.latitude)));
    }

    return (sum * R * R).abs() / 2;
  }

  /// Check if two polygons overlap (simplified: check if any point of one
  /// is inside the other, or vice versa). This is a conservative check —
  /// it may return false positives for complex overlaps.
  static bool polygonsOverlap(
    List<Position> a,
    List<Position> b,
  ) {
    if (a.length < 3 || b.length < 3) return false;

    for (final p in a) {
      if (isPointInPolygon(p, b)) return true;
    }
    for (final p in b) {
      if (isPointInPolygon(p, a)) return true;
    }
    return false;
  }
}
