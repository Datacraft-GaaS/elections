//! Iris + periocular biometric score fusion.

use serde::{Deserialize, Serialize};

/// Weights for combining iris and periocular modality scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionWeights {
    /// Weight for the iris score (default 0.7).
    pub iris: f32,
    /// Weight for the periocular score (default 0.3).
    pub periocular: f32,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            iris: 0.7,
            periocular: 0.3,
        }
    }
}

impl FusionWeights {
    /// Returns true when weights are individually in [0, 1] and sum to approximately 1.0.
    pub fn validate(&self) -> bool {
        let in_range = self.iris >= 0.0
            && self.iris <= 1.0
            && self.periocular >= 0.0
            && self.periocular <= 1.0;
        let sum_ok = (self.iris + self.periocular - 1.0).abs() < 1e-4;
        in_range && sum_ok
    }
}

/// Output of biometric score fusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricFusionResult {
    /// Combined score in [0, 1].
    pub fused_score: f32,
    /// Iris modality score, if available.
    pub iris_score: Option<f32>,
    /// Periocular modality score, if available.
    pub periocular_score: Option<f32>,
}

/// Fuses iris and periocular scores using configurable weights.
#[derive(Debug, Clone)]
pub struct BiometricFusion {
    pub weights: FusionWeights,
}

impl BiometricFusion {
    pub fn new(weights: FusionWeights) -> Self {
        Self { weights }
    }

    /// Fuse iris and periocular scores.
    ///
    /// When only one modality is available the other is down-weighted by 0.5
    /// and its contribution is replaced by the available score.
    pub fn fuse(
        &self,
        iris_score: Option<f32>,
        periocular_score: Option<f32>,
    ) -> BiometricFusionResult {
        let fused = match (iris_score, periocular_score) {
            (Some(i), Some(p)) => self.weights.iris * i + self.weights.periocular * p,
            (Some(i), None) => {
                // periocular missing: down-weight periocular contribution by 0.5
                let w_iris = self.weights.iris + self.weights.periocular * 0.5;
                w_iris * i
            }
            (None, Some(p)) => {
                // iris missing: down-weight iris contribution by 0.5
                let w_peri = self.weights.periocular + self.weights.iris * 0.5;
                w_peri * p
            }
            (None, None) => 0.0,
        };

        BiometricFusionResult {
            fused_score: fused.clamp(0.0, 1.0),
            iris_score,
            periocular_score,
        }
    }
}

impl Default for BiometricFusion {
    fn default() -> Self {
        Self::new(FusionWeights::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_validate() {
        assert!(FusionWeights::default().validate());
    }

    #[test]
    fn weights_sum_not_one_invalid() {
        let w = FusionWeights {
            iris: 0.6,
            periocular: 0.6,
        };
        assert!(!w.validate());
    }

    #[test]
    fn negative_weight_invalid() {
        let w = FusionWeights {
            iris: -0.1,
            periocular: 1.1,
        };
        assert!(!w.validate());
    }

    #[test]
    fn returns_iris_when_periocular_none() {
        let fusion = BiometricFusion::default();
        let result = fusion.fuse(Some(0.8), None);
        assert!(result.fused_score > 0.0);
        assert_eq!(result.iris_score, Some(0.8));
        assert_eq!(result.periocular_score, None);
    }

    #[test]
    fn returns_periocular_when_iris_none() {
        let fusion = BiometricFusion::default();
        let result = fusion.fuse(None, Some(0.6));
        assert!(result.fused_score > 0.0);
        assert_eq!(result.iris_score, None);
        assert_eq!(result.periocular_score, Some(0.6));
    }

    #[test]
    fn returns_zero_when_both_none() {
        let fusion = BiometricFusion::default();
        let result = fusion.fuse(None, None);
        assert_eq!(result.fused_score, 0.0);
    }

    #[test]
    fn fused_score_weighted_combination_when_both_present() {
        let w = FusionWeights {
            iris: 0.7,
            periocular: 0.3,
        };
        let fusion = BiometricFusion::new(w);
        let result = fusion.fuse(Some(1.0), Some(1.0));
        assert!((result.fused_score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fused_score_correct_weighted_combination() {
        let w = FusionWeights {
            iris: 0.7,
            periocular: 0.3,
        };
        let fusion = BiometricFusion::new(w);
        let result = fusion.fuse(Some(0.8), Some(0.6));
        let expected = 0.7 * 0.8 + 0.3 * 0.6;
        assert!(
            (result.fused_score - expected).abs() < 1e-5,
            "expected {expected}, got {}",
            result.fused_score
        );
    }

    #[test]
    fn fused_score_in_unit_interval() {
        let fusion = BiometricFusion::default();
        for (i, p) in [(0.0, 0.0), (1.0, 1.0), (0.5, 0.5), (0.9, 0.1)] {
            let r = fusion.fuse(Some(i), Some(p));
            assert!(
                r.fused_score >= 0.0 && r.fused_score <= 1.0,
                "out of range: {}",
                r.fused_score
            );
        }
    }

    #[test]
    fn iris_only_higher_than_zero() {
        let fusion = BiometricFusion::default();
        let result = fusion.fuse(Some(0.9), None);
        // With default weights iris=0.7, periocular=0.3: effective weight = 0.7+0.15=0.85
        let expected = (0.7 + 0.3 * 0.5) * 0.9;
        assert!((result.fused_score - expected).abs() < 1e-5);
    }

    #[test]
    fn periocular_only_higher_than_zero() {
        let fusion = BiometricFusion::default();
        let result = fusion.fuse(None, Some(0.9));
        let expected = (0.3 + 0.7 * 0.5) * 0.9;
        assert!((result.fused_score - expected).abs() < 1e-5);
    }
}
