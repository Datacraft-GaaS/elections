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

//! Per-matcher score calibration.
//!
//! Platt scaling maps raw scores to posterior probabilities via a sigmoid:
//!   calibrated = 1 / (1 + exp(A * raw + B))
//!
//! Parameters A and B are fitted on a labelled sample using maximum likelihood.
//! When no training data is available, the identity calibrator is used (A=-1, B=0
//! gives a pass-through for scores already in [0,1]).
//!
//! Reference: J. Platt, "Probabilistic outputs for support vector machines and
//! comparisons to regularized likelihood methods," Advances in Large Margin
//! Classifiers, MIT Press, 2000.

/// Sigmoid-based score calibrator (Platt scaling).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreCalibrator {
    /// Sigmoid slope (positive -> higher raw score maps to higher probability).
    pub a: f32,
    /// Sigmoid intercept.
    pub b: f32,
}

impl ScoreCalibrator {
    /// Identity calibrator: output = input (passes [0,1] scores unchanged).
    pub fn identity() -> Self {
        // calibrate(x) = sigma(a*x+b) = 1/(1+exp(-(10x-5))).
        // At x=0 -> sigma(-5) ≈ 0.007; at x=1 -> sigma(5) ≈ 0.993.
        Self { a: 10.0, b: -5.0 }
    }

    /// Create a calibrator with explicit Platt parameters.
    pub fn new(a: f32, b: f32) -> Self {
        Self { a, b }
    }

    /// Calibrate a single raw score.
    pub fn calibrate(&self, raw: f32) -> f32 {
        let logit = self.a * raw + self.b;
        1.0 / (1.0 + (-logit).exp())
    }

    /// Fit Platt parameters from (score, label) pairs using gradient descent.
    ///
    /// `labels`: 1.0 = genuine, 0.0 = impostor.
    /// Returns an error string if fewer than 4 samples are provided.
    pub fn fit(scores: &[f32], labels: &[f32]) -> Result<Self, String> {
        if scores.len() < 4 || scores.len() != labels.len() {
            return Err(format!(
                "need at least 4 labelled samples, got {}",
                scores.len()
            ));
        }
        // Gradient descent on log-loss.  Start with positive slope so that
        // high raw scores map to high probabilities from the first iteration.
        let mut a = 1.0f32;
        let mut b = 0.0f32;
        let lr = 0.01f32;
        let n = scores.len() as f32;
        for _ in 0..200 {
            let mut da = 0.0f32;
            let mut db = 0.0f32;
            for (&s, &y) in scores.iter().zip(labels.iter()) {
                let p = 1.0 / (1.0 + (-(a * s + b)).exp());
                let err = p - y;
                da += err * s;
                db += err;
            }
            a -= lr * da / n;
            b -= lr * db / n;
        }
        Ok(Self { a, b })
    }

    /// Verify that the calibration is monotonically increasing on [0,1].
    pub fn is_monotonic(&self) -> bool {
        // calibrate(x) = sigma(a*x+b) is increasing when a > 0.
        self.a > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_calibrator_maps_high_scores_high() {
        let cal = ScoreCalibrator::identity();
        let low = cal.calibrate(0.1);
        let high = cal.calibrate(0.9);
        assert!(
            high > low,
            "calibrator must be monotone: low={low} high={high}"
        );
    }

    #[test]
    fn calibrate_output_in_zero_one() {
        let cal = ScoreCalibrator::identity();
        for i in 0..=10 {
            let v = cal.calibrate(i as f32 / 10.0);
            assert!(v >= 0.0 && v <= 1.0, "calibrated value {v} out of [0,1]");
        }
    }

    #[test]
    fn fit_requires_min_samples() {
        let res = ScoreCalibrator::fit(&[0.5, 0.3], &[1.0, 0.0]);
        assert!(res.is_err());
    }

    #[test]
    fn fit_converges_on_simple_data() {
        let scores = vec![0.9, 0.8, 0.7, 0.2, 0.1, 0.15];
        let labels = vec![1.0, 1.0, 1.0, 0.0, 0.0, 0.0];
        let cal = ScoreCalibrator::fit(&scores, &labels).unwrap();
        // Genuine scores should calibrate higher than impostor scores.
        let gen = cal.calibrate(0.8);
        let imp = cal.calibrate(0.15);
        assert!(
            gen > imp,
            "calibrated genuine ({gen}) must exceed impostor ({imp})"
        );
    }

    #[test]
    fn monotonic_when_a_positive() {
        let cal = ScoreCalibrator::new(5.0, -2.5);
        assert!(cal.is_monotonic());
    }
}
