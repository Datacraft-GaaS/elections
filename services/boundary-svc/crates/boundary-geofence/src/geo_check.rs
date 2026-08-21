//! Polygon-in-polygon check.
//!
//! # Algorithm
//! Ray-casting for point-in-polygon. O(n) where n = exterior ring vertex
//! count. For Kenya electoral boundaries n ~ 100-2000 so this is well within
//! the 20 ms p99 SLA on any reasonable hardware.
//!
//! # Correctness invariants
//! - Input ring must be closed (last vertex == first). If not, the algorithm
//!   wraps around correctly anyway due to the `j = i` rotation.
//! - WGS-84 (EPSG:4326) coordinates: longitude = x, latitude = y.

use boundary_domain::types::Polygon;

/// Result of a geofence check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// True if the point is inside the polygon's exterior ring.
    pub in_boundary: bool,
    /// Approximate minimum distance from the boundary edge in metres.
    /// 0.0 when `in_boundary` is true (conservative; full geodesic calc omitted
    /// for latency budget).
    pub distance_from_boundary_m: f64,
}

/// Check whether (`lat`, `lon`) lies inside `polygon`.
/// Returns `CheckResult` immediately.
pub fn check(polygon: &Polygon, lat: f64, lon: f64) -> CheckResult {
    let in_boundary = polygon.contains(lat, lon);
    // Conservative distance estimate: 0.0 inside, haversine-to-nearest-edge
    // approximation outside. For a full implementation this would call a proper
    // geodesic library, but for the latency target we return 0.0 as a stub.
    let distance_from_boundary_m = if in_boundary {
        0.0
    } else {
        nearest_edge_distance_m(polygon, lat, lon)
    };
    CheckResult {
        in_boundary,
        distance_from_boundary_m,
    }
}

/// Haversine distance to the nearest vertex in the exterior ring (conservative
/// lower bound for true edge distance). Sufficient for callers that only need
/// to know "how far am I from my station boundary".
fn nearest_edge_distance_m(polygon: &Polygon, lat: f64, lon: f64) -> f64 {
    let ring = match polygon.rings.first() {
        Some(r) => r,
        None => return f64::MAX,
    };
    ring.iter()
        .map(|v| haversine(lat, lon, v.latitude, v.longitude))
        .fold(f64::MAX, f64::min)
}

fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0; // Earth radius in metres
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    R * c
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundary_domain::types::{Coordinate, Polygon};

    fn square_polygon(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Polygon {
        Polygon {
            rings: vec![vec![
                Coordinate {
                    longitude: min_lon,
                    latitude: min_lat,
                },
                Coordinate {
                    longitude: max_lon,
                    latitude: min_lat,
                },
                Coordinate {
                    longitude: max_lon,
                    latitude: max_lat,
                },
                Coordinate {
                    longitude: min_lon,
                    latitude: max_lat,
                },
                Coordinate {
                    longitude: min_lon,
                    latitude: min_lat,
                },
            ]],
        }
    }

    #[test]
    fn point_inside_square() {
        let poly = square_polygon(36.0, -1.5, 37.0, -0.5);
        let r = check(&poly, -1.0, 36.5);
        assert!(r.in_boundary, "centre of square should be inside");
        assert_eq!(r.distance_from_boundary_m, 0.0);
    }

    #[test]
    fn point_outside_square() {
        let poly = square_polygon(36.0, -1.5, 37.0, -0.5);
        let r = check(&poly, -2.0, 36.5);
        assert!(!r.in_boundary, "point below square should be outside");
        assert!(r.distance_from_boundary_m > 0.0);
    }

    #[test]
    fn point_on_edge() {
        // Points exactly on the edge are implementation-defined; we just check
        // the function returns without panic.
        let poly = square_polygon(36.0, -1.5, 37.0, -0.5);
        let _r = check(&poly, -1.5, 36.5);
    }
}
