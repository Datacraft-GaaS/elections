//! Core consensus logic: invoke all sub-matchers and fuse via weighted voting.
//!
//! Sub-matchers and their weights (calibrated on NIST FVC2004 data):
//!   1. Bozorth3  (abis-matcher)              weight 1.0
//!   2. MCC       (eel-abis-fp-mcc)           weight 1.2
//!   3. FingerCode texture (eel-abis-fp-texture)  weight 0.8
//!   4. Orientation field (eel-abis-fp-orientation) weight 0.7
//!   5. Skeleton graph (eel-abis-fp-skeleton)     weight 0.9

use abis_fusion_advanced::decision_level::weighted_vote;
use abis_matcher::{
    matcher::fingerprint_matcher::FingerprintMatcher, modality::fingerprint::FingerprintTemplate,
};
use eel_abis_fp_mcc::MccMatcher;
use eel_abis_fp_orientation::matcher::OrientationMatcher;
use eel_abis_fp_skeleton::{matcher::SkeletonMatcher, skeleton::BinaryImage};
use eel_abis_fp_texture::matcher::TextureMatcher;

use crate::{calibration::ScoreCalibrator, policy::MinAgreementPolicy, ConsensusError};

/// Per-sub-matcher result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubMatcherResult {
    /// Human-readable name.
    pub name: String,
    /// Raw (pre-calibration) score in [0.0, 1.0].
    pub raw_score: f32,
    /// Calibrated score in [0.0, 1.0].
    pub calibrated_score: f32,
    /// Individual match decision.
    pub decision: bool,
}

/// Aggregated consensus score.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsensusScore {
    /// Weighted-average fused score in [0.0, 1.0].
    pub fused: f32,
    /// Individual sub-matcher results.
    pub per_matcher: Vec<SubMatcherResult>,
    /// Final consensus decision.
    pub decision: bool,
    /// Human-readable explanation.
    pub explanation: String,
}

/// All inputs the consensus matcher may need.
///
/// Fields are optional because not all representations are always available.
/// Bozorth3 and MCC require `minutiae_*`; texture and orientation require `image_*`;
/// skeleton requires `binary_*`.
pub struct ConsensusInput<'a> {
    pub minutiae_probe: Option<&'a FingerprintTemplate>,
    pub minutiae_gallery: Option<&'a FingerprintTemplate>,
    /// Raw grayscale image bytes (row-major) for texture + orientation.
    pub image_probe: Option<&'a [u8]>,
    pub image_gallery: Option<&'a [u8]>,
    pub image_width: u32,
    pub image_height: u32,
    /// Pre-binarised pixel bytes for skeleton matching.
    pub binary_probe: Option<&'a [u8]>,
    pub binary_gallery: Option<&'a [u8]>,
}

/// Default per-sub-matcher thresholds.
const BOZORTH_THRESHOLD: f32 = 0.40;
const MCC_THRESHOLD: f32 = 0.45;
const TEXTURE_THRESHOLD: f32 = 0.50;
const ORIENT_THRESHOLD: f32 = 0.40;
const SKELETON_THRESHOLD: f32 = 0.35;

/// Default weights matching the array indices [bozorth, mcc, texture, orientation, skeleton].
const WEIGHTS: [f64; 5] = [1.0, 1.2, 0.8, 0.7, 0.9];

/// Multi-algorithm consensus matcher.
pub struct ConsensusMatcher {
    pub policy: MinAgreementPolicy,
    pub bozorth: FingerprintMatcher,
    pub mcc: MccMatcher,
    pub texture: TextureMatcher,
    pub orientation: OrientationMatcher,
    pub skeleton: SkeletonMatcher,
    pub calibrators: Vec<ScoreCalibrator>,
    pub weights: [f64; 5],
    /// Per-sub-matcher decision thresholds (same order as weights).
    pub thresholds: [f32; 5],
}

impl Default for ConsensusMatcher {
    fn default() -> Self {
        Self {
            policy: MinAgreementPolicy::default(),
            bozorth: FingerprintMatcher::default(),
            mcc: MccMatcher::default(),
            texture: TextureMatcher::default(),
            orientation: OrientationMatcher::default(),
            skeleton: SkeletonMatcher::default(),
            calibrators: (0..5).map(|_| ScoreCalibrator::identity()).collect(),
            weights: WEIGHTS,
            thresholds: [
                BOZORTH_THRESHOLD,
                MCC_THRESHOLD,
                TEXTURE_THRESHOLD,
                ORIENT_THRESHOLD,
                SKELETON_THRESHOLD,
            ],
        }
    }
}

impl ConsensusMatcher {
    /// Run all available sub-matchers and fuse via weighted voting.
    pub fn compare(&self, input: &ConsensusInput) -> Result<ConsensusScore, ConsensusError> {
        let mut results: Vec<SubMatcherResult> = Vec::with_capacity(5);

        // 1. Bozorth3.
        if let (Some(p), Some(g)) = (input.minutiae_probe, input.minutiae_gallery) {
            let raw = self.bozorth.raw_score(p, g);
            let cal = self.calibrators[0].calibrate(raw);
            results.push(SubMatcherResult {
                name: "bozorth3".into(),
                raw_score: raw,
                calibrated_score: cal,
                decision: cal >= self.thresholds[0],
            });
        }

        // 2. MCC.
        if let (Some(p), Some(g)) = (input.minutiae_probe, input.minutiae_gallery) {
            let raw = self.mcc.compare(p, g);
            let cal = self.calibrators[1].calibrate(raw);
            results.push(SubMatcherResult {
                name: "mcc".into(),
                raw_score: raw,
                calibrated_score: cal,
                decision: cal >= self.thresholds[1],
            });
        }

        // 3. FingerCode texture.
        if let (Some(pb), Some(gb)) = (input.image_probe, input.image_gallery) {
            let w = input.image_width;
            let h = input.image_height;
            match (
                eel_abis_fp_texture::GrayImage::new(w, h, pb.to_vec()),
                eel_abis_fp_texture::GrayImage::new(w, h, gb.to_vec()),
            ) {
                (Ok(pi), Ok(gi)) => match self.texture.compare(&pi, &gi) {
                    Ok(s) => {
                        let cal = self.calibrators[2].calibrate(s.fused);
                        results.push(SubMatcherResult {
                            name: "fingercode_texture".into(),
                            raw_score: s.fused,
                            calibrated_score: cal,
                            decision: cal >= self.thresholds[2],
                        });
                    }
                    Err(_) => {
                        results.push(zero_result("fingercode_texture", self.thresholds[2]));
                    }
                },
                _ => {}
            }
        }

        // 4. Orientation field.
        if let (Some(pb), Some(gb)) = (input.image_probe, input.image_gallery) {
            let w = input.image_width;
            let h = input.image_height;
            match (
                eel_abis_fp_orientation::GrayImage::new(w, h, pb.to_vec()),
                eel_abis_fp_orientation::GrayImage::new(w, h, gb.to_vec()),
            ) {
                (Ok(pi), Ok(gi)) => match self.orientation.compare(&pi, &gi) {
                    Ok(s) => {
                        let cal = self.calibrators[3].calibrate(s.fused);
                        results.push(SubMatcherResult {
                            name: "orientation_field".into(),
                            raw_score: s.fused,
                            calibrated_score: cal,
                            decision: cal >= self.thresholds[3],
                        });
                    }
                    Err(_) => {
                        results.push(zero_result("orientation_field", self.thresholds[3]));
                    }
                },
                _ => {}
            }
        }

        // 5. Skeleton.
        if let (Some(pb), Some(gb)) = (input.binary_probe, input.binary_gallery) {
            let w = input.image_width;
            let h = input.image_height;
            match (
                BinaryImage::new(w, h, pb.to_vec()),
                BinaryImage::new(w, h, gb.to_vec()),
            ) {
                (Ok(pi), Ok(gi)) => match self.skeleton.compare(&pi, &gi) {
                    Ok(s) => {
                        let cal = self.calibrators[4].calibrate(s.fused);
                        results.push(SubMatcherResult {
                            name: "skeleton_graph".into(),
                            raw_score: s.fused,
                            calibrated_score: cal,
                            decision: cal >= self.thresholds[4],
                        });
                    }
                    Err(_) => {
                        results.push(zero_result("skeleton_graph", self.thresholds[4]));
                    }
                },
                _ => {}
            }
        }

        if results.is_empty() {
            return Err(ConsensusError::NoMatchers);
        }

        // Policy check.
        let votes = results.iter().filter(|r| r.decision).count();
        let policy_pass = self.policy.check(votes, results.len());

        // Weighted fused score.
        let (w_sum, s_sum) =
            results
                .iter()
                .enumerate()
                .fold((0.0f32, 0.0f32), |(wt, ws), (i, r)| {
                    let w = self.weights.get(i).copied().unwrap_or(1.0) as f32;
                    (wt + w, ws + w * r.calibrated_score)
                });
        let fused = if w_sum > 0.0 { s_sum / w_sum } else { 0.0 };

        // Weighted decision vote.
        let decisions: Vec<bool> = results.iter().map(|r| r.decision).collect();
        let wts: Vec<f64> = (0..decisions.len())
            .map(|i| self.weights.get(i).copied().unwrap_or(1.0))
            .collect();
        let vote_decision = weighted_vote(&decisions, &wts).unwrap_or(false);
        let final_decision = policy_pass && vote_decision;

        let explanation = build_explanation(&results, votes, final_decision, fused);

        Ok(ConsensusScore {
            fused,
            per_matcher: results,
            decision: final_decision,
            explanation,
        })
    }
}

fn zero_result(name: &str, threshold: f32) -> SubMatcherResult {
    SubMatcherResult {
        name: name.into(),
        raw_score: 0.0,
        calibrated_score: 0.0,
        decision: 0.0f32 >= threshold,
    }
}

fn build_explanation(
    results: &[SubMatcherResult],
    votes: usize,
    decision: bool,
    fused: f32,
) -> String {
    let vote_str: String = results
        .iter()
        .map(|r| {
            format!(
                "{}={:.2}({})",
                r.name,
                r.calibrated_score,
                if r.decision { "Y" } else { "N" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "decision={} fused={:.3} votes={}/{} [{}]",
        if decision { "MATCH" } else { "NON-MATCH" },
        fused,
        votes,
        results.len(),
        vote_str
    )
}
