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

//! IREX report card: ROC curve, rank-N rates, and fairness slices.

use crate::{corpus::IrexCorpus, identify::IrexIdentifier, verify::IrexVerifier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single operating point on the ROC curve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocPoint {
    /// False Match Rate.
    pub fmr: f32,
    /// False Non-Match Rate.
    pub fnmr: f32,
    pub threshold: f32,
}

/// FMR/FNMR statistics for a demographic or sensor group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessSlice {
    pub group: String,
    pub fmr: f32,
    pub fnmr: f32,
    pub n_genuine: usize,
    pub n_impostor: usize,
}

/// Full IREX evaluation report card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportCard {
    /// ROC curve sampled at 20 threshold points.
    pub roc: Vec<RocPoint>,
    /// Rank-1 identification rate on genuine pairs.
    pub rank1_rate: f32,
    /// Rank-5 identification rate on genuine pairs.
    pub rank5_rate: f32,
    /// Per-group fairness slices.
    pub fairness: Vec<FairnessSlice>,
}

/// Compute FMR and FNMR at a given threshold for a set of genuine and impostor scores.
fn fmr_fnmr(genuine_scores: &[f32], impostor_scores: &[f32], threshold: f32) -> (f32, f32) {
    let fmr = if impostor_scores.is_empty() {
        0.0
    } else {
        impostor_scores.iter().filter(|&&s| s >= threshold).count() as f32
            / impostor_scores.len() as f32
    };
    let fnmr = if genuine_scores.is_empty() {
        0.0
    } else {
        genuine_scores.iter().filter(|&&s| s < threshold).count() as f32
            / genuine_scores.len() as f32
    };
    (fmr, fnmr)
}

impl ReportCard {
    /// Generate a full report card for `corpus` using `verifier` and `identifier`.
    pub fn generate(
        corpus: &IrexCorpus,
        verifier: &IrexVerifier,
        identifier: &IrexIdentifier,
    ) -> Self {
        // Collect global genuine and impostor scores.
        let genuine_scores: Vec<f32> = corpus
            .genuine_pairs()
            .iter()
            .map(|(a, b)| verifier.verify(a, b).score)
            .collect();

        let impostor_scores: Vec<f32> = corpus
            .impostor_pairs()
            .iter()
            .map(|(a, b)| verifier.verify(a, b).score)
            .collect();

        // ROC: 20 threshold points from 0.0 to 1.0 inclusive.
        let roc: Vec<RocPoint> = (0..20)
            .map(|i| {
                let t = i as f32 / 19.0;
                let (fmr, fnmr) = fmr_fnmr(&genuine_scores, &impostor_scores, t);
                RocPoint {
                    fmr,
                    fnmr,
                    threshold: t,
                }
            })
            .collect();

        // Rank-1 and rank-5 identification rates.
        let genuine_pairs = corpus.genuine_pairs();
        let entries = &corpus.entries;
        let (mut hit1, mut hit5) = (0usize, 0usize);

        for (probe, reference) in &genuine_pairs {
            let result = identifier.identify(probe, entries);
            let top_ids: Vec<&str> = result.rank_n.iter().map(|(id, _)| id.as_str()).collect();
            if top_ids.first().map_or(false, |id| *id == reference.id) {
                hit1 += 1;
            }
            if top_ids.iter().take(5).any(|id| *id == reference.id) {
                hit5 += 1;
            }
        }
        let n_genuine = genuine_pairs.len();
        let rank1_rate = if n_genuine == 0 {
            0.0
        } else {
            hit1 as f32 / n_genuine as f32
        };
        let rank5_rate = if n_genuine == 0 {
            0.0
        } else {
            hit5 as f32 / n_genuine as f32
        };

        // Fairness slices per group.
        let mut groups: HashMap<String, (Vec<f32>, Vec<f32>)> = HashMap::new();
        for entry in entries {
            groups.entry(entry.group.clone()).or_default();
        }

        // Genuine scores per group.
        for (a, b) in corpus.genuine_pairs() {
            let score = verifier.verify(a, b).score;
            if let Some((gen, _)) = groups.get_mut(&a.group) {
                gen.push(score);
            }
        }

        // Impostor scores per group.
        for (a, b) in corpus.impostor_pairs() {
            let score = verifier.verify(a, b).score;
            if let Some((_, imp)) = groups.get_mut(&a.group) {
                imp.push(score);
            }
        }

        // Use threshold = 0.5 for fairness slice metrics.
        let fairness_threshold = 0.5;
        let fairness: Vec<FairnessSlice> = {
            let mut slices: Vec<FairnessSlice> = groups
                .into_iter()
                .map(|(group, (gen, imp))| {
                    let (fmr, fnmr) = fmr_fnmr(&gen, &imp, fairness_threshold);
                    FairnessSlice {
                        group,
                        fmr,
                        fnmr,
                        n_genuine: gen.len(),
                        n_impostor: imp.len(),
                    }
                })
                .collect();
            slices.sort_by(|a, b| a.group.cmp(&b.group));
            slices
        };

        Self {
            roc,
            rank1_rate,
            rank5_rate,
            fairness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{corpus::CorpusEntry, identify::IrexIdentifier, verify::IrexVerifier, IrexCorpus};
    use eel_abis_iris_encode::{
        daugman_code::DaugmanCode, pie::PieCode, IrisTemplate, IRIS_CODE_BYTES,
    };

    fn synthetic_template(seed: u8) -> IrisTemplate {
        let code: [u8; IRIS_CODE_BYTES] = std::array::from_fn(|_| seed);
        let mask: [u8; IRIS_CODE_BYTES] = [0xFF; IRIS_CODE_BYTES];
        IrisTemplate {
            daugman: DaugmanCode { code, mask },
            pie: PieCode { code, mask },
            quality: seed as f32 / 255.0,
        }
    }

    fn make_entry(id: &str, group: &str, seed: u8) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            template: synthetic_template(seed),
            group: group.to_string(),
            quality: seed as f32 / 255.0,
        }
    }

    fn synthetic_corpus() -> IrexCorpus {
        let mut entries = Vec::new();
        for s in 0..10u8 {
            entries.push(make_entry(&format!("sub_{s}_a"), "g1", s * 10));
            entries.push(make_entry(&format!("sub_{s}_b"), "g1", s * 10 + 1));
        }
        IrexCorpus::new(entries).unwrap()
    }

    #[test]
    fn report_roc_has_20_points() {
        let corpus = synthetic_corpus();
        let verifier = IrexVerifier::new(0.5);
        let identifier = IrexIdentifier::new(5, 0.5);
        let card = ReportCard::generate(&corpus, &verifier, &identifier);
        assert_eq!(card.roc.len(), 20);
    }

    #[test]
    fn roc_fmr_fnmr_in_range() {
        let corpus = synthetic_corpus();
        let verifier = IrexVerifier::new(0.5);
        let identifier = IrexIdentifier::new(5, 0.5);
        let card = ReportCard::generate(&corpus, &verifier, &identifier);
        for point in &card.roc {
            assert!((0.0..=1.0).contains(&point.fmr), "fmr out of range");
            assert!((0.0..=1.0).contains(&point.fnmr), "fnmr out of range");
        }
    }

    #[test]
    fn rank1_rate_in_range() {
        let corpus = synthetic_corpus();
        let verifier = IrexVerifier::new(0.5);
        let identifier = IrexIdentifier::new(5, 0.5);
        let card = ReportCard::generate(&corpus, &verifier, &identifier);
        assert!((0.0..=1.0).contains(&card.rank1_rate));
    }

    #[test]
    fn fairness_slices_cover_all_groups() {
        let corpus = synthetic_corpus();
        let verifier = IrexVerifier::new(0.5);
        let identifier = IrexIdentifier::new(5, 0.5);
        let card = ReportCard::generate(&corpus, &verifier, &identifier);
        let groups: std::collections::HashSet<&str> =
            corpus.entries.iter().map(|e| e.group.as_str()).collect();
        let slice_groups: std::collections::HashSet<&str> =
            card.fairness.iter().map(|s| s.group.as_str()).collect();
        for g in &groups {
            assert!(
                slice_groups.contains(g),
                "group {g} missing from fairness slices"
            );
        }
    }

    #[test]
    fn report_generates_without_panic_on_10_entry_corpus() {
        let corpus = synthetic_corpus();
        let verifier = IrexVerifier::new(0.5);
        let identifier = IrexIdentifier::new(5, 0.5);
        // Should not panic.
        let _ = ReportCard::generate(&corpus, &verifier, &identifier);
    }
}
