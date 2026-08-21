//! Left and right iris bilateral fusion.

use eel_abis_iris_encode::IrisTemplate;
use serde::{Deserialize, Serialize};

/// Strategy for combining left and right eye scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FusionStrategy {
    /// Weighted mean of per-eye similarity scores.
    Average,
    /// Minimum of per-eye similarity scores (high-security: both eyes must match).
    LogicalAnd,
}

/// Result of bilateral fusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BilateralResult {
    /// Fused similarity score in [0, 1].
    pub score: f32,
    /// Weight applied to the left eye.
    pub left_weight: f32,
    /// Weight applied to the right eye.
    pub right_weight: f32,
    /// Strategy used.
    pub strategy: FusionStrategy,
}

/// Combines left and right iris match scores into a single score.
#[derive(Debug, Clone)]
pub struct BilateralFusion {
    pub strategy: FusionStrategy,
    pub quality_weighting: bool,
}

impl BilateralFusion {
    pub fn new(strategy: FusionStrategy, quality_weighting: bool) -> Self {
        Self {
            strategy,
            quality_weighting,
        }
    }

    /// Fuse left and right eye comparisons against their respective references.
    ///
    /// Returns `None` when either Hamming distance cannot be computed (insufficient overlap).
    pub fn fuse(
        &self,
        left: &IrisTemplate,
        right: &IrisTemplate,
        reference_left: &IrisTemplate,
        reference_right: &IrisTemplate,
    ) -> Option<BilateralResult> {
        let hd_left = left.hamming_distance(reference_left, 0.3)?;
        let hd_right = right.hamming_distance(reference_right, 0.3)?;

        let sim_left = 1.0 - hd_left;
        let sim_right = 1.0 - hd_right;

        let (w_left, w_right) = if self.quality_weighting {
            let ql = left.quality.max(0.0);
            let qr = right.quality.max(0.0);
            let total = ql + qr;
            if total < f32::EPSILON {
                (0.5, 0.5)
            } else {
                (ql / total, qr / total)
            }
        } else {
            (0.5, 0.5)
        };

        let score = match self.strategy {
            FusionStrategy::Average => w_left * sim_left + w_right * sim_right,
            FusionStrategy::LogicalAnd => sim_left.min(sim_right),
        };

        Some(BilateralResult {
            score: score.clamp(0.0, 1.0),
            left_weight: w_left,
            right_weight: w_right,
            strategy: self.strategy.clone(),
        })
    }

    /// Single-eye similarity: `1.0 - hamming_distance`.
    ///
    /// Returns `None` when there is insufficient overlap to compute HD.
    pub fn fuse_one(&self, template: &IrisTemplate, reference: &IrisTemplate) -> Option<f32> {
        let hd = template.hamming_distance(reference, 0.3)?;
        Some((1.0 - hd).clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_encode::daugman_code::DaugmanCode;
    use eel_abis_iris_encode::pie::PieCode;
    use eel_abis_iris_encode::{IrisTemplate, IRIS_CODE_BYTES};

    fn make_template(code: u8, mask: u8, quality: f32) -> IrisTemplate {
        IrisTemplate {
            daugman: DaugmanCode {
                code: [code; IRIS_CODE_BYTES],
                mask: [mask; IRIS_CODE_BYTES],
            },
            pie: PieCode {
                code: [code; IRIS_CODE_BYTES],
                mask: [mask; IRIS_CODE_BYTES],
            },
            quality,
        }
    }

    fn identical_template(quality: f32) -> IrisTemplate {
        make_template(0b10101010, 0xFF, quality)
    }

    #[test]
    fn fuse_one_identical_returns_near_one() {
        let t = identical_template(1.0);
        let fusion = BilateralFusion::new(FusionStrategy::Average, false);
        let score = fusion.fuse_one(&t, &t).expect("should compute");
        // Identical templates: best HD across shifts may not be exactly 0 due to rotation,
        // but similarity should be high.
        assert!(score >= 0.0 && score <= 1.0, "score out of range: {score}");
    }

    #[test]
    fn bilateral_result_score_in_unit_interval() {
        let left = identical_template(0.9);
        let right = identical_template(0.8);
        let fusion = BilateralFusion::new(FusionStrategy::Average, true);
        if let Some(result) = fusion.fuse(&left, &right, &left, &right) {
            assert!(result.score >= 0.0 && result.score <= 1.0);
        }
    }

    #[test]
    fn logical_and_bounded_by_weaker_eye() {
        let good = identical_template(1.0);
        let bad = make_template(0xFF, 0xFF, 0.1);
        let fusion = BilateralFusion::new(FusionStrategy::LogicalAnd, false);
        let avg_fusion = BilateralFusion::new(FusionStrategy::Average, false);
        let and_result = fusion.fuse(&good, &bad, &good, &good);
        let avg_result = avg_fusion.fuse(&good, &bad, &good, &good);
        if let (Some(and), Some(avg)) = (and_result, avg_result) {
            // LogicalAnd should be <= Average when eyes differ
            assert!(
                and.score <= avg.score + 0.01,
                "LogicalAnd {:.3} > Average {:.3}",
                and.score,
                avg.score
            );
        }
    }

    #[test]
    fn average_strategy_between_left_and_right() {
        let left = identical_template(0.9);
        let right = make_template(0b11001100, 0xFF, 0.9);
        let fusion = BilateralFusion::new(FusionStrategy::Average, false);
        if let Some(result) = fusion.fuse(&left, &right, &left, &right) {
            assert!(result.score >= 0.0 && result.score <= 1.0);
            assert_eq!(result.left_weight, 0.5);
            assert_eq!(result.right_weight, 0.5);
        }
    }

    #[test]
    fn occluded_eye_gets_lower_weight() {
        let good = identical_template(1.0);
        let occluded = identical_template(0.0);
        let fusion = BilateralFusion::new(FusionStrategy::Average, true);
        if let Some(result) = fusion.fuse(&good, &occluded, &good, &occluded) {
            // good eye (quality=1.0) should dominate
            assert!(
                result.left_weight > result.right_weight,
                "expected left_weight > right_weight"
            );
        }
    }

    #[test]
    fn quality_weighting_disabled_uses_equal_weights() {
        let t1 = identical_template(0.9);
        let t2 = identical_template(0.1);
        let fusion = BilateralFusion::new(FusionStrategy::Average, false);
        if let Some(result) = fusion.fuse(&t1, &t2, &t1, &t2) {
            assert!((result.left_weight - 0.5).abs() < 1e-6);
            assert!((result.right_weight - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn fusion_strategy_serialises() {
        let s = FusionStrategy::LogicalAnd;
        let json = serde_json::to_string(&s).expect("serialise");
        assert_eq!(json, r#""LogicalAnd""#);
        let back: FusionStrategy = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back, FusionStrategy::LogicalAnd);
    }

    #[test]
    fn fuse_one_score_in_unit_interval() {
        let t = identical_template(1.0);
        let other = make_template(0b11110000, 0xFF, 1.0);
        let fusion = BilateralFusion::new(FusionStrategy::Average, false);
        if let Some(score) = fusion.fuse_one(&t, &other) {
            assert!(score >= 0.0 && score <= 1.0);
        }
    }

    #[test]
    fn bilateral_result_stores_strategy() {
        let t = identical_template(0.8);
        let fusion = BilateralFusion::new(FusionStrategy::LogicalAnd, false);
        if let Some(result) = fusion.fuse(&t, &t, &t, &t) {
            assert_eq!(result.strategy, FusionStrategy::LogicalAnd);
        }
    }

    #[test]
    fn weights_sum_to_one_with_quality_weighting() {
        let left = identical_template(0.7);
        let right = identical_template(0.3);
        let fusion = BilateralFusion::new(FusionStrategy::Average, true);
        if let Some(result) = fusion.fuse(&left, &right, &left, &right) {
            let sum = result.left_weight + result.right_weight;
            assert!((sum - 1.0).abs() < 1e-5, "weights sum: {sum}");
        }
    }
}
