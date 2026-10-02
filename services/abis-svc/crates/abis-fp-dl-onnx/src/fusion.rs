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

//! Score-level fusion of DL, MCC, and Bozorth scores.
//!
//! Implements a weighted linear combination with calibration weights.
//! Weights should be set from a held-out calibration corpus; the defaults
//! are equal-weight (no a priori preference).

/// Calibration weights for three matchers.
#[derive(Debug, Clone)]
pub struct FusionWeights {
    /// Weight for the DL cosine similarity score.
    pub dl: f32,
    /// Weight for the MCC global similarity score.
    pub mcc: f32,
    /// Weight for the Bozorth3 score (normalised to [0, 1]).
    pub bozorth: f32,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            dl: 1.0 / 3.0,
            mcc: 1.0 / 3.0,
            bozorth: 1.0 / 3.0,
        }
    }
}

/// Fuse DL, MCC, and Bozorth scores into a single score in [0, 1].
///
/// Each score must already be in [0, 1].  The output is a weighted sum
/// renormalised to [0, 1].
pub fn fuse(dl: f32, mcc: f32, bozorth: f32, weights: &FusionWeights) -> f32 {
    let total = weights.dl + weights.mcc + weights.bozorth;
    let sum = weights.dl * dl + weights.mcc * mcc + weights.bozorth * bozorth;
    if total < 1e-8 {
        return 0.0;
    }
    (sum / total).clamp(0.0, 1.0)
}

/// Fit fusion weights from paired (dl, mcc, bozorth, genuine) observations.
/// Uses a simple grid search over weight ratios.
/// `samples`: Vec of (dl, mcc, bozorth, is_genuine).
pub fn fit_weights(samples: &[(f32, f32, f32, bool)]) -> FusionWeights {
    if samples.is_empty() {
        return FusionWeights::default();
    }
    let mut best_eer = f32::MAX;
    let mut best_w = FusionWeights::default();

    // 5-point grid (0.0, 0.25, 0.5, 0.75, 1.0) for each weight.
    for dw in [0u32, 1, 2, 3, 4] {
        for mw in [0u32, 1, 2, 3, 4] {
            for bw in [0u32, 1, 2, 3, 4] {
                let total = (dw + mw + bw) as f32;
                if total < 1e-4 {
                    continue;
                }
                let w = FusionWeights {
                    dl: dw as f32 / total,
                    mcc: mw as f32 / total,
                    bozorth: bw as f32 / total,
                };
                let scores: Vec<(f32, bool)> = samples
                    .iter()
                    .map(|&(d, m, b, g)| (fuse(d, m, b, &w), g))
                    .collect();
                let eer = estimate_eer(&scores);
                if eer < best_eer {
                    best_eer = eer;
                    best_w = w;
                }
            }
        }
    }
    best_w
}

/// Estimate Equal Error Rate from a list of (score, is_genuine) pairs.
fn estimate_eer(scores: &[(f32, bool)]) -> f32 {
    // Sort by threshold and sweep.
    let n_genuine = scores.iter().filter(|&&(_, g)| g).count();
    let n_impostor = scores.len() - n_genuine;
    if n_genuine == 0 || n_impostor == 0 {
        return 0.5;
    }
    let mut sorted: Vec<f32> = scores.iter().map(|&(s, _)| s).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut min_diff = f32::MAX;
    let mut eer = 0.5;
    for &t in &sorted {
        let fnmr = scores.iter().filter(|&&(s, g)| g && s < t).count() as f32 / n_genuine as f32;
        let fmr = scores.iter().filter(|&&(s, g)| !g && s >= t).count() as f32 / n_impostor as f32;
        let diff = (fnmr - fmr).abs();
        if diff < min_diff {
            min_diff = diff;
            eer = (fnmr + fmr) / 2.0;
        }
    }
    eer
}
