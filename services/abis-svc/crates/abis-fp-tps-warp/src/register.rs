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

//! Register probe minutiae to gallery via TPS.
//!
//! Process:
//! 1. Find initial correspondences by nearest-neighbour in (x,y,theta) space after
//!    a global rigid alignment (rotation + translation from the best single-pair alignment).
//! 2. Fit TPS to the correspondence set.
//! 3. Optionally iterate (IRLS-style) by re-weighting and re-fitting.

use crate::{Minutia, Point, TpsError, TpsWarp};

/// Maximum translation tolerance for initial correspondences (pixels).
const TRANS_TOL: f64 = 50.0;
/// Maximum angle difference for initial correspondences (radians, ~20 deg).
const ANGLE_TOL: f64 = 0.35;
/// Minimum correspondences needed for a TPS fit.
const MIN_CORR: usize = 3;

/// Register probe minutiae to gallery minutiae via TPS.
///
/// Returns a fitted TPS warp, or None when fewer than MIN_CORR correspondences are found.
pub fn register_minutiae(
    probe: &[Minutia],
    gallery: &[Minutia],
) -> Result<Option<TpsWarp>, TpsError> {
    if probe.is_empty() || gallery.is_empty() {
        return Ok(None);
    }

    // Step 1: rigid alignment from best single-pair.
    let (best_tx, best_ty, best_rot) = best_rigid_alignment(probe, gallery);

    // Step 2: collect correspondences under rigid alignment.
    let corr = find_correspondences(probe, gallery, best_tx, best_ty, best_rot);
    if corr.len() < MIN_CORR {
        return Ok(None);
    }

    let src: Vec<Point> = corr.iter().map(|&(pi, _)| probe[pi].position()).collect();
    let dst: Vec<Point> = corr.iter().map(|&(_, gi)| gallery[gi].position()).collect();

    // Step 3: fit TPS.
    let tps = TpsWarp::fit(&src, &dst)?;

    // Step 4: one round of IRLS -- re-weight by residual.
    let src2: Vec<Point> = src.clone();
    let dst2: Vec<Point> = dst.clone();
    let weights: Vec<f64> = src2
        .iter()
        .zip(dst2.iter())
        .map(|(s, d)| {
            let (wx, wy) = tps.apply(s.x, s.y);
            let residual = ((wx - d.x).powi(2) + (wy - d.y).powi(2)).sqrt();
            1.0 / (1.0 + residual / 5.0) // Huber-like weight
        })
        .collect();

    // Rebuild source/target with only high-weight points.
    let min_w = 0.5f64;
    let src3: Vec<Point> = src2
        .iter()
        .zip(weights.iter())
        .filter(|(_, &w)| w >= min_w)
        .map(|(p, _)| *p)
        .collect();
    let dst3: Vec<Point> = dst2
        .iter()
        .zip(weights.iter())
        .filter(|(_, &w)| w >= min_w)
        .map(|(p, _)| *p)
        .collect();

    if src3.len() >= MIN_CORR {
        let refined = TpsWarp::fit(&src3, &dst3)?;
        Ok(Some(refined))
    } else {
        Ok(Some(tps))
    }
}

/// Find the best rigid alignment (tx, ty, rotation) from all probe-gallery minutia pairs.
/// Returns (best_tx, best_ty, best_rot_rad).
fn best_rigid_alignment(probe: &[Minutia], gallery: &[Minutia]) -> (f64, f64, f64) {
    let mut best_count = 0usize;
    let mut best = (0.0f64, 0.0f64, 0.0f64);

    for pi in probe {
        for gi in gallery {
            let rot = gi.theta - pi.theta;
            let cos_r = rot.cos();
            let sin_r = rot.sin();
            let tx = gi.x - (pi.x * cos_r - pi.y * sin_r);
            let ty = gi.y - (pi.x * sin_r + pi.y * cos_r);
            let count = count_matches_under_rigid(probe, gallery, tx, ty, rot);
            if count > best_count {
                best_count = count;
                best = (tx, ty, rot);
            }
        }
    }
    best
}

fn count_matches_under_rigid(
    probe: &[Minutia],
    gallery: &[Minutia],
    tx: f64,
    ty: f64,
    rot: f64,
) -> usize {
    let cos_r = rot.cos();
    let sin_r = rot.sin();
    probe
        .iter()
        .filter(|p| {
            let px = p.x * cos_r - p.y * sin_r + tx;
            let py = p.x * sin_r + p.y * cos_r + ty;
            gallery.iter().any(|g| {
                let dx = px - g.x;
                let dy = py - g.y;
                dx * dx + dy * dy <= TRANS_TOL * TRANS_TOL
                    && angle_diff(p.theta + rot, g.theta) <= ANGLE_TOL
            })
        })
        .count()
}

/// Find index pairs (probe_i, gallery_i) for correspondences under the rigid alignment.
fn find_correspondences(
    probe: &[Minutia],
    gallery: &[Minutia],
    tx: f64,
    ty: f64,
    rot: f64,
) -> Vec<(usize, usize)> {
    let cos_r = rot.cos();
    let sin_r = rot.sin();
    let mut result: Vec<(usize, usize)> = Vec::new();
    let mut used_gallery: Vec<bool> = vec![false; gallery.len()];

    for (pi, p) in probe.iter().enumerate() {
        let px = p.x * cos_r - p.y * sin_r + tx;
        let py = p.x * sin_r + p.y * cos_r + ty;
        let mut best_dist = f64::MAX;
        let mut best_gi = None;
        for (gi, g) in gallery.iter().enumerate() {
            if used_gallery[gi] {
                continue;
            }
            let dx = px - g.x;
            let dy = py - g.y;
            let d2 = dx * dx + dy * dy;
            if d2 <= TRANS_TOL * TRANS_TOL
                && angle_diff(p.theta + rot, g.theta) <= ANGLE_TOL
                && d2 < best_dist
            {
                best_dist = d2;
                best_gi = Some(gi);
            }
        }
        if let Some(gi) = best_gi {
            used_gallery[gi] = true;
            result.push((pi, gi));
        }
    }
    result
}

fn angle_diff(a: f64, b: f64) -> f64 {
    let mut d = (a - b).abs() % std::f64::consts::TAU;
    if d > std::f64::consts::PI {
        d = std::f64::consts::TAU - d;
    }
    if d > std::f64::consts::PI / 2.0 {
        d = std::f64::consts::PI - d;
    }
    d
}
