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

//! Quality policy gate: accept/reject an iris sample based on metric thresholds.
//!
//! Reference: ISO/IEC 29794-6:2021, Section 7 (quality thresholds).

use crate::quality::QualityReport;
use serde::{Deserialize, Serialize};

/// Error from policy configuration.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("min_iris_diameter_px must be positive, got {0}")]
    InvalidDiameter(f32),
    #[error("dilation range invalid: min {min} >= max {max}")]
    InvalidDilationRange { min: f32, max: f32 },
    #[error("max_occlusion_pct must be in [0, 1], got {0}")]
    InvalidOcclusion(f32),
    #[error("max_gaze_deg must be positive, got {0}")]
    InvalidGaze(f32),
}

/// Policy verdict for a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Accept,
    Reject,
}

/// Rejection reasons (may be multiple).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionReasons {
    pub iris_diameter_too_small: bool,
    pub dilation_out_of_range: bool,
    pub occlusion_too_high: bool,
    pub gaze_too_large: bool,
    pub quality_too_low: bool,
}

impl RejectionReasons {
    pub fn any(&self) -> bool {
        self.iris_diameter_too_small
            || self.dilation_out_of_range
            || self.occlusion_too_high
            || self.gaze_too_large
            || self.quality_too_low
    }
}

/// Quality policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPolicy {
    /// Minimum limbus diameter in pixels (ISO 29794-6: minimum 200 px).
    pub min_iris_diameter_px: f32,
    /// Minimum dilation ratio (pupil/limbus).
    pub min_dilation: f32,
    /// Maximum dilation ratio.
    pub max_dilation: f32,
    /// Maximum occlusion fraction (0 = none, 1 = fully occluded).
    pub max_occlusion_frac: f32,
    /// Maximum gaze angle in degrees.
    pub max_gaze_deg: f32,
    /// Minimum overall quality score.
    pub min_quality: f32,
}

impl Default for QualityPolicy {
    fn default() -> Self {
        Self {
            min_iris_diameter_px: 200.0,
            min_dilation: 0.2,
            max_dilation: 0.6,
            max_occlusion_frac: 0.30,
            max_gaze_deg: 15.0,
            min_quality: 0.5,
        }
    }
}

impl QualityPolicy {
    /// Validate policy configuration.
    ///
    /// # Errors
    /// Returns `PolicyError` when any threshold is out of range.
    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.min_iris_diameter_px <= 0.0 {
            return Err(PolicyError::InvalidDiameter(self.min_iris_diameter_px));
        }
        if self.min_dilation >= self.max_dilation {
            return Err(PolicyError::InvalidDilationRange {
                min: self.min_dilation,
                max: self.max_dilation,
            });
        }
        if !(0.0..=1.0).contains(&self.max_occlusion_frac) {
            return Err(PolicyError::InvalidOcclusion(self.max_occlusion_frac));
        }
        if self.max_gaze_deg <= 0.0 {
            return Err(PolicyError::InvalidGaze(self.max_gaze_deg));
        }
        Ok(())
    }

    /// Evaluate the policy against a quality report.
    ///
    /// `limbus_radius_px` is the segmented limbus radius in pixels.
    pub fn evaluate(
        &self,
        report: &QualityReport,
        limbus_radius_px: f32,
    ) -> (Verdict, RejectionReasons) {
        let reasons = RejectionReasons {
            iris_diameter_too_small: (2.0 * limbus_radius_px) < self.min_iris_diameter_px,
            dilation_out_of_range: report.dilation < self.min_dilation
                || report.dilation > self.max_dilation,
            occlusion_too_high: report.occlusion_fraction > self.max_occlusion_frac,
            gaze_too_large: report.gaze_angle_deg > self.max_gaze_deg,
            quality_too_low: report.overall < self.min_quality,
        };
        let verdict = if reasons.any() {
            Verdict::Reject
        } else {
            Verdict::Accept
        };
        (verdict, reasons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_report() -> QualityReport {
        QualityReport {
            sharpness: 0.8,
            contrast: 0.7,
            dilation: 0.4,
            iris_sclera_ratio: 0.85,
            gaze_angle_deg: 3.0,
            occlusion_fraction: 0.1,
            overall: 0.75,
        }
    }

    #[test]
    fn good_sample_accepted() {
        let policy = QualityPolicy::default();
        let (verdict, _) = policy.evaluate(&good_report(), 120.0);
        assert_eq!(verdict, Verdict::Accept);
    }

    #[test]
    fn small_iris_rejected() {
        let policy = QualityPolicy::default();
        // radius = 80px -> diameter = 160px < 200px minimum
        let (verdict, reasons) = policy.evaluate(&good_report(), 80.0);
        assert_eq!(verdict, Verdict::Reject);
        assert!(reasons.iris_diameter_too_small);
    }

    #[test]
    fn excessive_dilation_rejected() {
        let policy = QualityPolicy::default();
        let mut report = good_report();
        report.dilation = 0.8; // > 0.6 max
        let (verdict, reasons) = policy.evaluate(&report, 120.0);
        assert_eq!(verdict, Verdict::Reject);
        assert!(reasons.dilation_out_of_range);
    }

    #[test]
    fn excessive_occlusion_rejected() {
        let policy = QualityPolicy::default();
        let mut report = good_report();
        report.occlusion_fraction = 0.5; // > 0.30 max
        let (verdict, reasons) = policy.evaluate(&report, 120.0);
        assert_eq!(verdict, Verdict::Reject);
        assert!(reasons.occlusion_too_high);
    }

    #[test]
    fn excessive_gaze_rejected() {
        let policy = QualityPolicy::default();
        let mut report = good_report();
        report.gaze_angle_deg = 20.0; // > 15 deg max
        let (verdict, reasons) = policy.evaluate(&report, 120.0);
        assert_eq!(verdict, Verdict::Reject);
        assert!(reasons.gaze_too_large);
    }

    #[test]
    fn low_quality_rejected() {
        let policy = QualityPolicy::default();
        let mut report = good_report();
        report.overall = 0.2; // < 0.5 min
        let (verdict, reasons) = policy.evaluate(&report, 120.0);
        assert_eq!(verdict, Verdict::Reject);
        assert!(reasons.quality_too_low);
    }

    #[test]
    fn default_policy_validates() {
        QualityPolicy::default()
            .validate()
            .expect("default policy should be valid");
    }

    #[test]
    fn invalid_dilation_range_rejected() {
        let policy = QualityPolicy {
            min_dilation: 0.6,
            max_dilation: 0.2,
            ..Default::default()
        };
        let err = policy.validate().unwrap_err();
        assert!(matches!(err, PolicyError::InvalidDilationRange { .. }));
    }

    #[test]
    fn policy_composition_all_criteria_checked() {
        // Verify that multiple violations are all reported.
        let policy = QualityPolicy::default();
        let report = QualityReport {
            sharpness: 0.1,
            contrast: 0.1,
            dilation: 0.8, // too high
            iris_sclera_ratio: 0.5,
            gaze_angle_deg: 20.0,    // too high
            occlusion_fraction: 0.5, // too high
            overall: 0.1,            // too low
        };
        let (verdict, reasons) = policy.evaluate(&report, 80.0); // diameter too small
        assert_eq!(verdict, Verdict::Reject);
        assert!(reasons.iris_diameter_too_small);
        assert!(reasons.dilation_out_of_range);
        assert!(reasons.occlusion_too_high);
        assert!(reasons.gaze_too_large);
        assert!(reasons.quality_too_low);
    }
}
