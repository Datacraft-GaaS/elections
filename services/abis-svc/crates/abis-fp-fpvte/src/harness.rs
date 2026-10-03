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

//! FpVTE + PFT-III evaluation harness.
//!
//! Runs the evaluation tracks defined in:
//! - P. Grother et al., "NIST Fingerprint Vendor Technology Evaluation 2003,"
//!   NISTIR 7123, 2004.
//! - P. Grother, M. Indovina, "Proprietary Fingerprint Template Evaluation III,"
//!   NIST IR 8307, 2020.
//!
//! Tracks implemented:
//! 1. 1:1 Verification (probes x gallery), FMR/FNMR ROC curve.
//! 2. 1:N Identification, rank-N accuracy.
//! 3. Per-demographic fairness slicing.

use crate::{
    corpus::Corpus,
    matcher_trait::FpMatcher,
    report::{FairnessSlice, RankAccuracy, Report, RocPoint},
    FpVteError,
};
use std::path::Path;

/// Configuration for a harness run.
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Number of ROC threshold steps.
    pub roc_steps: usize,
    /// Maximum rank for rank-N identification accuracy.
    pub max_rank: usize,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            roc_steps: 100,
            max_rank: 10,
        }
    }
}

/// FpVTE evaluation harness.
pub struct FpVteHarness {
    config: HarnessConfig,
}

impl FpVteHarness {
    pub fn new(config: HarnessConfig) -> Self {
        Self { config }
    }

    /// Run the full evaluation suite on a corpus.
    ///
    /// Returns a `Report`; optionally writes JSON + CSV + text to `output_dir`.
    pub fn run(
        &self,
        corpus: &Corpus,
        matcher: &dyn FpMatcher,
        output_dir: Option<&Path>,
    ) -> Result<Report, FpVteError> {
        let subject_ids = corpus.subject_ids();
        if subject_ids.len() < 2 {
            return Err(FpVteError::InsufficientCorpus {
                needed: 2,
                got: subject_ids.len(),
            });
        }

        // Build probe set (first acquisition per subject) and gallery (second).
        let probes: Vec<_> = subject_ids
            .iter()
            .filter_map(|id| {
                corpus
                    .entries_for_subject(id)
                    .into_iter()
                    .find(|e| e.acquisition == 0)
                    .cloned()
            })
            .collect();
        let gallery: Vec<_> = subject_ids
            .iter()
            .filter_map(|id| {
                corpus
                    .entries_for_subject(id)
                    .into_iter()
                    .find(|e| e.acquisition == 1)
                    .cloned()
            })
            .collect();

        if gallery.is_empty() {
            return Err(FpVteError::InsufficientCorpus { needed: 1, got: 0 });
        }

        // Compute all probe-gallery scores.
        // (score, is_genuine)
        let mut score_pairs: Vec<(f32, bool)> = Vec::new();
        for probe in &probes {
            for gal in &gallery {
                let score = matcher.compare(&probe.template, &gal.template)?;
                let genuine = probe.subject_id == gal.subject_id;
                score_pairs.push((score, genuine));
            }
        }

        // ROC curve.
        let roc = self.compute_roc(&score_pairs);

        // EER.
        let eer = compute_eer(&roc);

        // FMR at FNMR=0.01.
        let fmr_at_fnmr_001 = fmr_at_fnmr(&roc, 0.01);

        // Rank-N identification accuracy.
        let rank_accuracies = self.compute_rank_accuracies(&probes, &gallery, matcher)?;
        let rank1_accuracy = rank_accuracies.first().map(|r| r.accuracy).unwrap_or(0.0);

        // Fairness slices (if demographics available).
        let fairness_slices = self.compute_fairness_slices(&probes, &gallery, matcher)?;

        let report = Report {
            matcher_name: matcher.name().to_string(),
            n_probes: probes.len(),
            n_gallery: gallery.len(),
            roc,
            rank_accuracies,
            fairness_slices,
            eer,
            fmr_at_fnmr_001,
            rank1_accuracy,
        };

        if let Some(dir) = output_dir {
            report.write_json(dir)?;
            report.write_roc_csv(dir)?;
            report.write_summary(dir)?;
        }

        Ok(report)
    }

    /// Compute the ROC curve from (score, is_genuine) pairs.
    fn compute_roc(&self, pairs: &[(f32, bool)]) -> Vec<RocPoint> {
        let n_genuine = pairs.iter().filter(|&&(_, g)| g).count();
        let n_impostor = pairs.len() - n_genuine;
        if n_genuine == 0 || n_impostor == 0 {
            return vec![];
        }

        let mut thresholds: Vec<f32> = pairs.iter().map(|&(s, _)| s).collect();
        thresholds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        thresholds.dedup_by(|a, b| (*a - *b).abs() < 1e-7);

        let step = (thresholds.len() / self.config.roc_steps).max(1);
        let mut roc = Vec::new();
        for i in (0..thresholds.len()).step_by(step) {
            let t = thresholds[i];
            let fnmr = pairs.iter().filter(|&&(s, g)| g && s < t).count() as f32 / n_genuine as f32;
            let fmr =
                pairs.iter().filter(|&&(s, g)| !g && s >= t).count() as f32 / n_impostor as f32;
            roc.push(RocPoint {
                threshold: t,
                fmr,
                fnmr,
            });
        }
        roc
    }

    /// Compute rank-N identification accuracy.
    fn compute_rank_accuracies(
        &self,
        probes: &[crate::corpus::CorpusEntry],
        gallery: &[crate::corpus::CorpusEntry],
        matcher: &dyn FpMatcher,
    ) -> Result<Vec<RankAccuracy>, FpVteError> {
        let mut rank_hits = vec![0usize; self.config.max_rank];
        let n = probes.len();
        if n == 0 {
            return Ok(vec![]);
        }

        for probe in probes {
            let mut scores: Vec<(f32, &str)> = Vec::with_capacity(gallery.len());
            for gal in gallery {
                let score = matcher.compare(&probe.template, &gal.template)?;
                scores.push((score, &gal.subject_id));
            }
            // Sort descending by score.
            scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

            for rank in 0..self.config.max_rank {
                if rank < scores.len() && scores[rank].1 == probe.subject_id {
                    // Hit at this rank and all lower ranks.
                    for hit in &mut rank_hits[rank..] {
                        *hit += 1;
                    }
                    break;
                }
            }
        }

        Ok((0..self.config.max_rank)
            .map(|r| RankAccuracy {
                rank: r + 1,
                accuracy: rank_hits[r] as f32 / n as f32,
            })
            .collect())
    }

    /// Compute per-demographic fairness slices.
    fn compute_fairness_slices(
        &self,
        probes: &[crate::corpus::CorpusEntry],
        gallery: &[crate::corpus::CorpusEntry],
        matcher: &dyn FpMatcher,
    ) -> Result<Vec<FairnessSlice>, FpVteError> {
        // Collect unique sex labels.
        let sex_labels: Vec<String> = {
            let mut labels: Vec<String> = probes
                .iter()
                .filter_map(|e| e.demographics.sex.clone())
                .chain(gallery.iter().filter_map(|e| e.demographics.sex.clone()))
                .collect();
            labels.sort_unstable();
            labels.dedup();
            labels
        };

        if sex_labels.is_empty() {
            // No demographic labels: return empty slice list.
            return Ok(vec![]);
        }

        let mut slices = Vec::new();
        for label in &sex_labels {
            let slice_probes: Vec<_> = probes
                .iter()
                .filter(|e| e.demographics.sex.as_deref() == Some(label))
                .collect();
            let slice_gallery: Vec<_> = gallery
                .iter()
                .filter(|e| e.demographics.sex.as_deref() == Some(label))
                .collect();

            if slice_probes.is_empty() || slice_gallery.is_empty() {
                continue;
            }

            let mut pairs: Vec<(f32, bool)> = Vec::new();
            for probe in &slice_probes {
                for gal in &slice_gallery {
                    let score = matcher.compare(&probe.template, &gal.template)?;
                    pairs.push((score, probe.subject_id == gal.subject_id));
                }
            }
            let roc = self.compute_roc(&pairs);
            let (fmr, fnmr) = fmr_fnmr_at_threshold(&roc, 0.01);
            slices.push(FairnessSlice {
                label: label.clone(),
                fmr_at_001: fmr,
                fnmr_at_001: fnmr,
            });
        }
        Ok(slices)
    }
}

/// Compute EER from an ROC curve.
fn compute_eer(roc: &[RocPoint]) -> f32 {
    if roc.is_empty() {
        return 0.5;
    }
    let mut best_eer = 0.5f32;
    let mut min_diff = f32::MAX;
    for pt in roc {
        let diff = (pt.fmr - pt.fnmr).abs();
        if diff < min_diff {
            min_diff = diff;
            best_eer = (pt.fmr + pt.fnmr) / 2.0;
        }
    }
    best_eer.clamp(0.0, 1.0)
}

/// FMR when FNMR is closest to target.
fn fmr_at_fnmr(roc: &[RocPoint], target_fnmr: f32) -> Option<f32> {
    roc.iter()
        .min_by(|a, b| {
            (a.fnmr - target_fnmr)
                .abs()
                .partial_cmp(&(b.fnmr - target_fnmr).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|pt| pt.fmr)
}

/// (FMR, FNMR) at the threshold closest to `target_threshold`.
fn fmr_fnmr_at_threshold(roc: &[RocPoint], target_threshold: f32) -> (f32, f32) {
    if let Some(pt) = roc.iter().min_by(|a, b| {
        (a.threshold - target_threshold)
            .abs()
            .partial_cmp(&(b.threshold - target_threshold).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        (pt.fmr, pt.fnmr)
    } else {
        (1.0, 1.0)
    }
}

impl Default for FpVteHarness {
    fn default() -> Self {
        Self::new(HarnessConfig::default())
    }
}
