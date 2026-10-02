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

//! TPS warp wrapper around a base minutiae similarity function.
//!
//! Warps probe minutiae via the fitted TPS before comparing to gallery,
//! improving scores on distorted prints by correcting non-rigid deformation.

use crate::{register::register_minutiae, Minutia, TpsError};

/// A simple score between two aligned minutiae sets using count-overlap.
///
/// Returns a normalised score in [0.0, 1.0].
pub fn minutiae_overlap_score(
    probe: &[Minutia],
    gallery: &[Minutia],
    trans_tol: f64,
    angle_tol: f64,
) -> f32 {
    if probe.is_empty() || gallery.is_empty() {
        return 0.0;
    }
    let mut matched = 0usize;
    let mut used = vec![false; gallery.len()];
    for p in probe {
        for (gi, g) in gallery.iter().enumerate() {
            if used[gi] {
                continue;
            }
            let dx = p.x - g.x;
            let dy = p.y - g.y;
            let d2 = dx * dx + dy * dy;
            let da = angle_diff(p.theta, g.theta);
            if d2 <= trans_tol * trans_tol && da <= angle_tol {
                matched += 1;
                used[gi] = true;
                break;
            }
        }
    }
    let denom = probe.len().max(gallery.len()) as f32;
    matched as f32 / denom
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

/// Configuration for TPS warp matcher.
#[derive(Debug, Clone)]
pub struct TpsWarpMatcher {
    /// Translational tolerance (pixels) for counting minutia matches.
    pub trans_tol: f64,
    /// Angular tolerance (radians) for counting minutia matches.
    pub angle_tol: f64,
    /// Decision threshold.
    pub threshold: f32,
}

impl Default for TpsWarpMatcher {
    fn default() -> Self {
        Self {
            trans_tol: 20.0,
            angle_tol: 0.26,
            threshold: 0.4,
        }
    }
}

/// Result from TPS warp matching.
#[derive(Debug, Clone)]
pub struct TpsMatchResult {
    /// Score without TPS warp applied (plain rigid alignment only).
    pub score_before: f32,
    /// Score after TPS warp.
    pub score_after: f32,
    /// Whether TPS improved the score.
    pub improved: bool,
}

impl TpsWarpMatcher {
    pub fn new(trans_tol: f64, angle_tol: f64, threshold: f32) -> Self {
        Self {
            trans_tol,
            angle_tol,
            threshold,
        }
    }

    /// Compare probe and gallery minutiae, optionally applying TPS registration.
    pub fn compare(
        &self,
        probe: &[Minutia],
        gallery: &[Minutia],
    ) -> Result<TpsMatchResult, TpsError> {
        // Score without TPS.
        let score_before = minutiae_overlap_score(probe, gallery, self.trans_tol, self.angle_tol);

        // Try to register via TPS.
        let warped_probe: Vec<Minutia> = match register_minutiae(probe, gallery)? {
            Some(tps) => probe.iter().map(|m| m.warped(&tps)).collect(),
            None => probe.to_vec(),
        };

        let score_after =
            minutiae_overlap_score(&warped_probe, gallery, self.trans_tol, self.angle_tol);

        // Use the better of the two scores.
        let improved = score_after > score_before;

        Ok(TpsMatchResult {
            score_before,
            score_after,
            improved,
        })
    }

    /// Best score (max of before and after warp).
    pub fn best_score(result: &TpsMatchResult) -> f32 {
        result.score_before.max(result.score_after)
    }
}
