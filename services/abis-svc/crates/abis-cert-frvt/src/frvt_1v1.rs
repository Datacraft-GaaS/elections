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

//! FRVT 1:1 Verification track submission.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Frvt1v1Error {
    #[error("template must not be empty")]
    EmptyTemplate,
    #[error("FMR {fmr:.6} exceeds target {target:.6}")]
    FmrExceedsTarget { fmr: f64, target: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPair {
    pub enrolment_template: Vec<u8>,
    pub probe_template: Vec<u8>,
    pub ground_truth_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frvt1v1Report {
    pub tar_at_far_1e3: f64,
    pub tar_at_far_1e4: f64,
    pub tar_at_far_1e6: f64,
    pub genuine_pairs: u64,
    pub impostor_pairs: u64,
}

pub struct Frvt1v1Submission {
    algorithm_id: String,
    version: String,
}

impl Frvt1v1Submission {
    pub fn new(algorithm_id: &str, version: &str) -> Result<Self, Frvt1v1Error> {
        if algorithm_id.is_empty() || version.is_empty() {
            return Err(Frvt1v1Error::EmptyTemplate);
        }
        Ok(Self {
            algorithm_id: algorithm_id.to_owned(),
            version: version.to_owned(),
        })
    }

    /// Runs the 1:1 pipeline over test pairs and computes TAR@FAR operating points.
    ///
    /// # Errors
    /// Returns an error if FMR exceeds the target.
    pub fn run(
        &self,
        pairs: &[VerificationPair],
        target_fmr: f64,
    ) -> Result<Frvt1v1Report, Frvt1v1Error> {
        let genuine: Vec<_> = pairs.iter().filter(|p| p.ground_truth_match).collect();
        let impostor: Vec<_> = pairs.iter().filter(|p| !p.ground_truth_match).collect();

        // Stub: compute toy similarity = byte-level overlap fraction.
        let genuine_scores: Vec<f64> = genuine
            .iter()
            .map(|p| toy_sim(&p.enrolment_template, &p.probe_template))
            .collect();
        let impostor_scores: Vec<f64> = impostor
            .iter()
            .map(|p| toy_sim(&p.enrolment_template, &p.probe_template))
            .collect();

        let tar_1e3 = tar_at_far(&genuine_scores, &impostor_scores, 1e-3);
        let tar_1e4 = tar_at_far(&genuine_scores, &impostor_scores, 1e-4);
        let tar_1e6 = tar_at_far(&genuine_scores, &impostor_scores, 1e-6);

        // FMR check requires impostor pairs to be meaningful.
        if !impostor_scores.is_empty() {
            let threshold = tar_1e3;
            let fmr = impostor_scores.iter().filter(|&&s| s >= threshold).count() as f64
                / impostor_scores.len() as f64;
            if fmr > target_fmr {
                return Err(Frvt1v1Error::FmrExceedsTarget {
                    fmr,
                    target: target_fmr,
                });
            }
        }

        Ok(Frvt1v1Report {
            tar_at_far_1e3: tar_1e3,
            tar_at_far_1e4: tar_1e4,
            tar_at_far_1e6: tar_1e6,
            genuine_pairs: genuine.len() as u64,
            impostor_pairs: impostor.len() as u64,
        })
    }

    pub fn algorithm_id(&self) -> &str {
        &self.algorithm_id
    }
    pub fn version(&self) -> &str {
        &self.version
    }
}

fn toy_sim(a: &[u8], b: &[u8]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let matching = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
    matching as f64 / a.len().max(b.len()) as f64
}

fn tar_at_far(genuine: &[f64], impostor: &[f64], far_target: f64) -> f64 {
    if impostor.is_empty() || genuine.is_empty() {
        return 0.0;
    }
    // Find threshold that achieves approximately `far_target`.
    let mut impostor_sorted = impostor.to_vec();
    impostor_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((1.0 - far_target) * impostor_sorted.len() as f64) as usize;
    let threshold = impostor_sorted.get(idx).copied().unwrap_or(1.0);
    let accepted = genuine.iter().filter(|&&s| s >= threshold).count();
    accepted as f64 / genuine.len() as f64
}
