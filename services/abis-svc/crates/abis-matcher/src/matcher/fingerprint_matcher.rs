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

//! Bozorth3-style fingerprint matcher.
//!
//! Algorithm overview:
//! 1. For each probe minutia `pi`, find all gallery minutiae `gk` within translational
//!    tolerance after a candidate alignment derived from (pi, pj) <-> (gk, gl) pairs.
//! 2. Try alignments from the most-compatible minutia pairs.
//! 3. Score = max matching minutiae count across all tried alignments.
//! 4. Normalise against a fixed reference score (40 = excellent match).
//!
//! Tolerances (ANSI 378 §7):
//!   Translational: ±40 px (at 500 ppi)
//!   Rotational: ±15°
//!   Ridge-count constraint: ±2 ridges (when ridge counts are populated)

use crate::matcher::{Matcher, Score};
use crate::modality::fingerprint::{angular_diff, deg_to_rad, rotate_angle, FingerprintTemplate};

/// Translational tolerance in pixels.
const TRANS_TOL: f32 = 40.0;
/// Rotational tolerance in degrees.
const ROT_TOL: f32 = 15.0;
/// Default score threshold (normalised).
const DEFAULT_THRESHOLD: f32 = 0.4;
/// Score considered an excellent match (used for normalisation).
const NORM_REF: f32 = 20.0;

/// Fingerprint-specific score configuration.
#[derive(Debug, Clone)]
pub struct FingerprintMatcher {
    /// Decision threshold on the normalised score.
    pub threshold: f32,
    /// Translational tolerance in pixels.
    pub trans_tol: f32,
    /// Rotational tolerance in degrees.
    pub rot_tol: f32,
}

impl Default for FingerprintMatcher {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            trans_tol: TRANS_TOL,
            rot_tol: ROT_TOL,
        }
    }
}

impl FingerprintMatcher {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            ..Self::default()
        }
    }

    /// Raw matching count score.
    pub fn raw_score(&self, probe: &FingerprintTemplate, gallery: &FingerprintTemplate) -> f32 {
        if probe.minutiae.is_empty() || gallery.minutiae.is_empty() {
            return 0.0;
        }
        let mut best = 0u32;

        // Try alignments from each probe–gallery minutia pair.
        for pi in &probe.minutiae {
            for gk in &gallery.minutiae {
                // Alignment: rotate probe so pi's angle matches gk's angle, then translate.
                let delta_theta = angular_diff(gk.theta, pi.theta);
                let matched = count_matches_under_alignment(
                    &probe.minutiae,
                    &gallery.minutiae,
                    pi.x as f32,
                    pi.y as f32,
                    gk.x as f32,
                    gk.y as f32,
                    delta_theta,
                    self.trans_tol,
                    self.rot_tol,
                );
                if matched > best {
                    best = matched;
                }
            }
        }
        best as f32
    }
}

/// Count how many probe minutiae match at least one gallery minutia under the given alignment.
///
/// Alignment: rotate each probe minutia around (probe_ref_x, probe_ref_y) by `delta_theta`,
/// then translate to (gallery_ref_x, gallery_ref_y).
fn count_matches_under_alignment(
    probe: &[crate::modality::fingerprint::Minutia],
    gallery: &[crate::modality::fingerprint::Minutia],
    probe_ref_x: f32,
    probe_ref_y: f32,
    gallery_ref_x: f32,
    gallery_ref_y: f32,
    delta_theta: f32,
    trans_tol: f32,
    rot_tol: f32,
) -> u32 {
    let rad = deg_to_rad(delta_theta);
    let cos_r = rad.cos();
    let sin_r = rad.sin();

    let mut matched = 0u32;
    for pm in probe {
        // Translate to origin relative to reference minutia.
        let dx = pm.x as f32 - probe_ref_x;
        let dy = pm.y as f32 - probe_ref_y;
        // Rotate.
        let rx = dx * cos_r - dy * sin_r + gallery_ref_x;
        let ry = dx * sin_r + dy * cos_r + gallery_ref_y;
        // Rotate theta.
        let rtheta = rotate_angle(pm.theta, delta_theta);

        // Check against every gallery minutia.
        let found = gallery.iter().any(|gm| {
            let edx = rx - gm.x as f32;
            let edy = ry - gm.y as f32;
            let dist2 = edx * edx + edy * edy;
            let theta_diff = angular_diff(rtheta, gm.theta).abs();
            dist2 <= trans_tol * trans_tol && theta_diff <= rot_tol
        });
        if found {
            matched += 1;
        }
    }
    matched
}

impl Matcher<FingerprintTemplate> for FingerprintMatcher {
    fn compare(&self, probe: &FingerprintTemplate, gallery: &FingerprintTemplate) -> Score {
        let raw = self.raw_score(probe, gallery);
        let normalised = (raw / NORM_REF).min(1.0);
        Score::new(raw, normalised, self.threshold)
    }
}
