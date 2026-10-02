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

//! Per-vendor Platt scaling so scores from heterogeneous vendors are comparable
//! before consensus voting.
//!
//! Platt, J. "Probabilistic outputs for support vector machines and comparisons
//! to regularized likelihood methods," Advances in Large Margin Classifiers,
//! MIT Press, 1999.
//!
//! Given a raw vendor score s, the calibrated posterior is:
//!   P(match | s) = sigmoid(A * s + B)
//! where A, B are fit per vendor on a held-out enrolment-vs-impostor set.

use serde::{Deserialize, Serialize};

use crate::PeakError;

/// Platt sigmoid calibrator for one vendor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlattCalibrator {
    /// Slope (A). Negative for well-behaved classifiers (higher score = more likely match).
    pub a: f64,
    /// Intercept (B).
    pub b: f64,
}

impl PlattCalibrator {
    pub fn new(a: f64, b: f64) -> Self {
        Self { a, b }
    }

    /// Map a raw score in [0, 1] to a calibrated posterior in (0, 1).
    ///
    /// Returns an error if the input is outside [0, 1].
    pub fn calibrate(&self, raw_score: f64) -> Result<f64, PeakError> {
        if !(0.0..=1.0).contains(&raw_score) {
            return Err(PeakError::Calibration(format!(
                "raw_score {raw_score} out of range [0, 1]"
            )));
        }
        Ok(sigmoid(self.a * raw_score + self.b))
    }

    /// Default calibrator centering sigmoid at raw score 0.5.
    ///
    /// With a=1, b=-0.5: sigmoid(0.5 - 0.5) = sigmoid(0) = 0.5, so the
    /// posterior=0.5 decision boundary aligns with raw score 0.5.
    pub fn identity() -> Self {
        Self { a: 1.0, b: -0.5 }
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Calibrators for all three vote slots (internal, vendor_a, vendor_b).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorCalibrators {
    pub internal: PlattCalibrator,
    pub vendor_a: PlattCalibrator,
    pub vendor_b: PlattCalibrator,
}

impl Default for VendorCalibrators {
    fn default() -> Self {
        Self {
            internal: PlattCalibrator::identity(),
            vendor_a: PlattCalibrator::identity(),
            vendor_b: PlattCalibrator::identity(),
        }
    }
}

impl VendorCalibrators {
    pub fn calibrate_all(
        &self,
        internal: f64,
        vendor_a: f64,
        vendor_b: f64,
    ) -> Result<(f64, f64, f64), PeakError> {
        let ci = self.internal.calibrate(internal)?;
        let ca = self.vendor_a.calibrate(vendor_a)?;
        let cb = self.vendor_b.calibrate(vendor_b)?;
        Ok((ci, ca, cb))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_maps_half_to_half() {
        let c = PlattCalibrator::identity();
        let out = c.calibrate(0.5).unwrap();
        // sigmoid(1*0.5 - 0.5) = sigmoid(0) = 0.5
        assert!((out - 0.5).abs() < 0.001);
    }

    #[test]
    fn calibrate_rejects_out_of_range() {
        let c = PlattCalibrator::identity();
        assert!(c.calibrate(-0.1).is_err());
        assert!(c.calibrate(1.1).is_err());
    }

    #[test]
    fn calibration_monotone_increasing() {
        // With a > 0 the calibrated output must be monotone increasing in raw score.
        let c = PlattCalibrator::new(5.0, -2.5);
        let scores: Vec<f64> = (0..=10).map(|i| i as f64 / 10.0).collect();
        let cal: Vec<f64> = scores.iter().map(|&s| c.calibrate(s).unwrap()).collect();
        for w in cal.windows(2) {
            assert!(w[1] >= w[0], "not monotone: {} < {}", w[1], w[0]);
        }
    }

    #[test]
    fn calibrate_all_returns_triple() {
        let vc = VendorCalibrators::default();
        let (ci, ca, cb) = vc.calibrate_all(0.3, 0.5, 0.7).unwrap();
        assert!(ci > 0.0 && ci < 1.0);
        assert!(ca > 0.0 && ca < 1.0);
        assert!(cb > 0.0 && cb < 1.0);
    }
}
