//! Local ridge structure descriptors for SourceAFIS matching.
//!
//! For each minutia, a neighbourhood of nearby minutiae is collected.  Each
//! neighbour contributes an "edge" record encoding relative distance, relative
//! direction, and ridge count between the two minutiae.
//!
//! The descriptor is invariant to global translation and rotation because all
//! quantities are expressed relative to the reference minutia's position and
//! direction.
//!
//! Reference: <https://sourceafis.machinezoo.com/algorithm#edge-table>

use crate::minutia::SourceAfisMinutia;
use crate::params::SourceAfisParams;
use std::f32::consts::PI;

/// One edge in the local ridge structure of a minutia.
#[derive(Debug, Clone)]
pub struct StructureEdge {
    /// Index of the neighbour minutia.
    pub neighbour_idx: usize,
    /// Distance to neighbour (pixels).
    pub distance: f32,
    /// Angle from this minutia to the neighbour, relative to this minutia's direction.
    pub relative_angle: f32,
    /// Ridge count between this minutia and the neighbour (estimated from distance
    /// and typical ridge period of ~0.5 mm at 500 ppi = ~10 px).
    pub ridge_count: u8,
}

/// Estimated number of ridges between two minutiae given their pixel distance.
/// Uses a nominal ridge period of 10 pixels at 500 ppi (ISO 19794-2 typical).
fn estimate_ridge_count(distance: f32) -> u8 {
    // Ridge period ~10 px at 500 ppi.
    let count = (distance / 10.0).round() as u8;
    count.min(50)
}

/// Normalise an angle to (-PI, PI].
fn wrap_angle(a: f32) -> f32 {
    let mut a = a % (2.0 * PI);
    if a > PI {
        a -= 2.0 * PI;
    } else if a <= -PI {
        a += 2.0 * PI;
    }
    a
}

/// Build the local ridge structure (edge table) for all minutiae.
pub fn build_edge_table(
    minutiae: &[SourceAfisMinutia],
    params: &SourceAfisParams,
) -> Vec<Vec<StructureEdge>> {
    let r2 = params.neighbour_radius_px * params.neighbour_radius_px;
    minutiae
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut edges: Vec<StructureEdge> = minutiae
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .filter_map(|(j, n)| {
                    let dx = n.x - m.x;
                    let dy = n.y - m.y;
                    let dist2 = dx * dx + dy * dy;
                    if dist2 > r2 {
                        return None;
                    }
                    let distance = dist2.sqrt();
                    // Angle from m to n, relative to m's direction.
                    let abs_angle = dy.atan2(dx);
                    let relative_angle = wrap_angle(abs_angle - m.direction);
                    let ridge_count = estimate_ridge_count(distance);
                    Some(StructureEdge {
                        neighbour_idx: j,
                        distance,
                        relative_angle,
                        ridge_count,
                    })
                })
                .collect();
            // Sort by distance so closest neighbours are first.
            edges.sort_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            edges
        })
        .collect()
}

/// Compare two edges for similarity, returning a score in [0, 1].
/// Returns None when the edges are outside tolerance.
pub fn edge_similarity(
    a: &StructureEdge,
    b: &StructureEdge,
    params: &SourceAfisParams,
    radius: f32,
) -> Option<f32> {
    let dist_diff = (a.distance - b.distance).abs() / radius;
    if dist_diff > params.max_distance_diff_frac {
        return None;
    }
    let angle_diff = wrap_angle(a.relative_angle - b.relative_angle).abs();
    if angle_diff > params.max_angle_diff {
        return None;
    }
    let ridge_diff = a.ridge_count.abs_diff(b.ridge_count);
    if ridge_diff > params.max_ridge_count_diff {
        return None;
    }
    // Simple linear score: 1 at perfect match, decays to 0 at tolerance boundary.
    let s = (1.0 - dist_diff / params.max_distance_diff_frac)
        * (1.0 - angle_diff / params.max_angle_diff)
        * (1.0 - ridge_diff as f32 / (params.max_ridge_count_diff as f32 + 1.0));
    Some(s.max(0.0))
}
