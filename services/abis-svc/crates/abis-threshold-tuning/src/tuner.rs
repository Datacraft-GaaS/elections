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

//! ROC-based threshold selection with bootstrap confidence intervals.

use crate::domain::{BootstrapCi, LabelledPair, SimulateResult};
use crate::error::ThresholdError;
use tracing::debug;

/// A single ROC operating point.
#[derive(Debug, Clone)]
pub struct RocPoint {
    pub threshold: f64,
    pub fmr: f64,
    pub fnmr: f64,
}

/// Compute the full ROC curve for a set of labelled pairs.
/// Returns points sorted ascending by threshold.
pub fn compute_roc(pairs: &[LabelledPair]) -> Vec<RocPoint> {
    if pairs.is_empty() {
        return vec![];
    }
    let mut scores: Vec<f64> = pairs.iter().map(|p| p.score).collect();
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    scores.dedup_by(|a, b| (*a - *b).abs() < 1e-10);

    let n_genuine: usize = pairs.iter().filter(|p| p.genuine).count();
    let n_impostor: usize = pairs.len() - n_genuine;

    if n_genuine == 0 || n_impostor == 0 {
        return vec![];
    }

    scores
        .iter()
        .map(|&t| {
            let fa = pairs.iter().filter(|p| !p.genuine && p.score >= t).count();
            let fr = pairs.iter().filter(|p| p.genuine && p.score < t).count();
            RocPoint {
                threshold: t,
                fmr: fa as f64 / n_impostor as f64,
                fnmr: fr as f64 / n_genuine as f64,
            }
        })
        .collect()
}

/// Select the threshold that achieves <= target_fmr.
/// Returns the operating point with highest threshold whose FMR is <= target_fmr.
/// If no point satisfies the constraint, returns the point with smallest FMR.
pub fn select_threshold(roc: &[RocPoint], target_fmr: f64) -> Result<&RocPoint, ThresholdError> {
    if !(0.0..1.0).contains(&target_fmr) {
        return Err(ThresholdError::InvalidTargetFmr(target_fmr));
    }
    let candidates: Vec<&RocPoint> = roc.iter().filter(|p| p.fmr <= target_fmr).collect();
    if let Some(best) = candidates.into_iter().max_by(|a, b| {
        a.threshold
            .partial_cmp(&b.threshold)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        return Ok(best);
    }
    roc.iter()
        .min_by(|a, b| {
            a.fmr
                .partial_cmp(&b.fmr)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or(ThresholdError::InsufficientData { min: 2, got: 0 })
}

/// Stateless threshold tuner.
pub struct ThresholdTuner {
    pub n_bootstrap: u32,
}

impl Default for ThresholdTuner {
    fn default() -> Self {
        Self { n_bootstrap: 1000 }
    }
}

impl ThresholdTuner {
    pub fn new(n_bootstrap: u32) -> Self {
        Self { n_bootstrap }
    }

    /// Compute threshold + bootstrap 95% CI for the given labelled pairs.
    pub fn tune(
        &self,
        pairs: &[LabelledPair],
        target_fmr: f64,
    ) -> Result<SimulateResult, ThresholdError> {
        const MIN_PAIRS: usize = 20;
        if pairs.len() < MIN_PAIRS {
            return Err(ThresholdError::InsufficientData {
                min: MIN_PAIRS,
                got: pairs.len(),
            });
        }
        if !(0.0..1.0).contains(&target_fmr) {
            return Err(ThresholdError::InvalidTargetFmr(target_fmr));
        }

        let roc = compute_roc(pairs);
        let point = select_threshold(&roc, target_fmr)?;
        let threshold = point.threshold;

        debug!(threshold, target_fmr, "selected threshold");

        let ci = self.bootstrap_ci(pairs, target_fmr, threshold)?;

        Ok(SimulateResult {
            threshold,
            achieved_fmr: point.fmr,
            achieved_fnmr: point.fnmr,
            ci,
            n_pairs: pairs.len(),
        })
    }

    fn bootstrap_ci(
        &self,
        pairs: &[LabelledPair],
        target_fmr: f64,
        nominal_threshold: f64,
    ) -> Result<BootstrapCi, ThresholdError> {
        let n = pairs.len();
        let mut thresholds: Vec<f64> = Vec::with_capacity(self.n_bootstrap as usize);

        // Deterministic LCG for reproducible CI without external rand crate.
        let mut lcg: u64 = 0xdeadbeef_cafef00d;
        let lcg_next = |state: &mut u64| -> u64 {
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *state
        };

        for _ in 0..self.n_bootstrap {
            let mut sample: Vec<LabelledPair> = Vec::with_capacity(n);
            for _ in 0..n {
                let idx = (lcg_next(&mut lcg) as usize) % n;
                sample.push(pairs[idx].clone());
            }
            let roc = compute_roc(&sample);
            if roc.is_empty() {
                thresholds.push(nominal_threshold);
                continue;
            }
            let t = select_threshold(&roc, target_fmr)
                .map(|p| p.threshold)
                .unwrap_or(nominal_threshold);
            thresholds.push(t);
        }

        thresholds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let lo_idx = ((self.n_bootstrap as f64) * 0.025) as usize;
        let hi_idx = ((self.n_bootstrap as f64) * 0.975) as usize;
        let lo_idx = lo_idx.min(thresholds.len() - 1);
        let hi_idx = hi_idx.min(thresholds.len() - 1);

        Ok(BootstrapCi {
            lower: thresholds[lo_idx],
            upper: thresholds[hi_idx],
            n_bootstrap: self.n_bootstrap,
        })
    }
}
