//! K-plet descriptor: ridge-count-aware nearest-neighbour descriptor per minutia.
//!
//! Reference: N. K. Ratha, R. M. Bolle, V. D. Pandit, V. Vaish,
//! "Robust fingerprint authentication using local structural similarity,"
//! IEEE Workshop on AVSPA, 2000.
//!
//! For each minutia m_i, find its K nearest neighbours. For each neighbour m_j record:
//!   rho:          Euclidean distance from m_i to m_j
//!   phi:          angle from m_i's orientation to the direction (m_i -> m_j)
//!   theta_diff:   difference in minutia orientations
//!   ridge_count:  number of ridges between m_i and m_j
//!
//! Matching: two descriptors match if their sorted tuple-sets have sufficient overlap.

use abis_matcher::modality::fingerprint::{FingerprintTemplate, Minutia};
use serde::{Deserialize, Serialize};

use crate::count::{count_ridges, OrientationField};

/// Number of nearest neighbours per minutia.
pub const K: usize = 5;

/// One entry in a K-plet descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPlet {
    /// Euclidean distance to neighbour (pixels).
    pub rho: f32,
    /// Angle from source minutia orientation to direction of neighbour (degrees).
    pub phi: f32,
    /// Orientation difference between source and neighbour minutiae (degrees).
    pub theta_diff: f32,
    /// Ridge count between the two minutiae.
    pub ridge_count: u8,
}

/// K-plet descriptor for one minutia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPletDescriptor {
    /// Index of the source minutia in its template.
    pub minutia_idx: usize,
    /// Sorted list of K (or fewer) neighbour tuples.
    pub tuples: Vec<KPlet>,
}

/// Build K-plet descriptors for all minutiae in `template`.
///
/// `orientation_field` is used for ridge counting; pass an empty field to use
/// the geometric fallback in `count_ridges`.
pub fn build_kplets(
    template: &FingerprintTemplate,
    orientation_field: &OrientationField,
) -> Vec<KPletDescriptor> {
    let minutiae = &template.minutiae;
    minutiae
        .iter()
        .enumerate()
        .map(|(i, mi)| build_one_kplet(i, mi, minutiae, orientation_field))
        .collect()
}

fn build_one_kplet(
    i: usize,
    mi: &Minutia,
    minutiae: &[Minutia],
    orientation_field: &OrientationField,
) -> KPletDescriptor {
    // Compute (distance, neighbour_index) for all other minutiae.
    let mut neighbours: Vec<(f32, usize)> = minutiae
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != i)
        .map(|(j, mj)| {
            let dx = mj.x as f32 - mi.x as f32;
            let dy = mj.y as f32 - mi.y as f32;
            (dx * dx + dy * dy, j)
        })
        .collect();

    // Sort by ascending distance.
    neighbours.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    neighbours.truncate(K);

    let mi_theta_rad = mi.theta * std::f32::consts::PI / 180.0;

    let tuples = neighbours
        .into_iter()
        .map(|(dist_sq, j)| {
            let mj = &minutiae[j];
            let rho = dist_sq.sqrt();
            let dx = mj.x as f32 - mi.x as f32;
            let dy = mj.y as f32 - mi.y as f32;
            // Angle from mi's orientation to direction of mj.
            let dir = dy.atan2(dx);
            let phi = (dir - mi_theta_rad).rem_euclid(2.0 * std::f32::consts::PI) * 180.0
                / std::f32::consts::PI;
            let theta_diff = angular_diff(mi.theta, mj.theta);
            let ridge_count = count_ridges(mi, mj, orientation_field);
            KPlet {
                rho,
                phi,
                theta_diff,
                ridge_count,
            }
        })
        .collect();

    KPletDescriptor {
        minutia_idx: i,
        tuples,
    }
}

/// Angular difference in [0, 180] (symmetric, orientation convention).
#[inline]
fn angular_diff(a: f32, b: f32) -> f32 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 {
        360.0 - d
    } else {
        d
    }
}
