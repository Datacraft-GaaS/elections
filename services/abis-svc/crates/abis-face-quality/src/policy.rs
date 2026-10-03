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

//! QualityPolicy: configurable acceptance gates for face quality metrics.
//!
//! Reference: ISO/IEC 29794-5:2010 clause 5 (minimum requirements for
//! face images submitted for enrolment).

use crate::metrics::PoseAngles;
use crate::report::QualityReport;
use serde::{Deserialize, Serialize};

/// Reason a policy check failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyViolation {
    SharpnessTooLow { score: f32, threshold: f32 },
    ResolutionTooLow { iod_px: f32, min_px: f32 },
    PoseYawTooLarge { yaw_deg: f32, max_deg: f32 },
    PosePitchTooLarge { pitch_deg: f32, max_deg: f32 },
    PoseRollTooLarge { roll_deg: f32, max_deg: f32 },
    IlluminationTooLow { score: f32, threshold: f32 },
    OcclusionTooHigh { occlusion: f32, max_occlusion: f32 },
    ExpressionNotNeutral { score: f32, threshold: f32 },
    OverallTooLow { score: f32, threshold: f32 },
}

/// Configurable quality acceptance policy.
///
/// Default values implement the ISO/IEC 29794-5 minimum standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPolicy {
    /// Minimum Laplacian-variance sharpness score [0, 1].
    pub min_sharpness: f32,
    /// Minimum interocular distance in pixels (ISO 19794-5: >= 60 for ABIS).
    pub min_iod_px: f32,
    /// Maximum head yaw in degrees.
    pub max_yaw_deg: f32,
    /// Maximum head pitch in degrees.
    pub max_pitch_deg: f32,
    /// Maximum head roll in degrees.
    pub max_roll_deg: f32,
    /// Minimum illumination score.
    pub min_illumination: f32,
    /// Maximum occlusion fraction [0, 1] (15% = 0.15 per ISO 29794-5).
    pub max_occlusion: f32,
    /// Minimum expression neutrality score.
    pub min_expression: f32,
    /// Minimum overall score.
    pub min_overall: f32,
}

impl Default for QualityPolicy {
    /// Default values are the ISO/IEC 29794-5 minimum requirements.
    fn default() -> Self {
        Self {
            min_sharpness: 0.2,
            min_iod_px: 60.0,
            max_yaw_deg: 10.0,
            max_pitch_deg: 15.0,
            max_roll_deg: 8.0,
            min_illumination: 0.3,
            max_occlusion: 0.15,
            min_expression: 0.4,
            min_overall: 0.5,
        }
    }
}

impl QualityPolicy {
    /// Evaluate a quality report against this policy.
    ///
    /// Returns all violations found (empty = accepted).
    pub fn evaluate(&self, report: &QualityReport, iod_px: f32) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();

        if report.sharpness < self.min_sharpness {
            violations.push(PolicyViolation::SharpnessTooLow {
                score: report.sharpness,
                threshold: self.min_sharpness,
            });
        }
        if iod_px < self.min_iod_px {
            violations.push(PolicyViolation::ResolutionTooLow {
                iod_px,
                min_px: self.min_iod_px,
            });
        }
        if report.pose_angles.yaw.abs() > self.max_yaw_deg {
            violations.push(PolicyViolation::PoseYawTooLarge {
                yaw_deg: report.pose_angles.yaw,
                max_deg: self.max_yaw_deg,
            });
        }
        if report.pose_angles.pitch.abs() > self.max_pitch_deg {
            violations.push(PolicyViolation::PosePitchTooLarge {
                pitch_deg: report.pose_angles.pitch,
                max_deg: self.max_pitch_deg,
            });
        }
        if report.pose_angles.roll.abs() > self.max_roll_deg {
            violations.push(PolicyViolation::PoseRollTooLarge {
                roll_deg: report.pose_angles.roll,
                max_deg: self.max_roll_deg,
            });
        }
        if report.illumination < self.min_illumination {
            violations.push(PolicyViolation::IlluminationTooLow {
                score: report.illumination,
                threshold: self.min_illumination,
            });
        }
        if report.occlusion > self.max_occlusion {
            violations.push(PolicyViolation::OcclusionTooHigh {
                occlusion: report.occlusion,
                max_occlusion: self.max_occlusion,
            });
        }
        if report.expression < self.min_expression {
            violations.push(PolicyViolation::ExpressionNotNeutral {
                score: report.expression,
                threshold: self.min_expression,
            });
        }
        if report.overall < self.min_overall {
            violations.push(PolicyViolation::OverallTooLow {
                score: report.overall,
                threshold: self.min_overall,
            });
        }
        violations
    }

    /// Returns true when the report passes all policy gates.
    pub fn accept(&self, report: &QualityReport, iod_px: f32) -> bool {
        self.evaluate(report, iod_px).is_empty()
    }
}

/// Build a default report for policy testing with a specific angle set.
// One parameter per ISO 29794-5 metric; bundling them would obscure the metric set.
#[allow(clippy::too_many_arguments)]
pub fn make_report(
    sharpness: f32,
    resolution: f32,
    pose_angles: PoseAngles,
    illumination: f32,
    uniformity: f32,
    occlusion: f32,
    expression: f32,
    gaze: f32,
) -> QualityReport {
    let pose = crate::metrics::pose_quality(&pose_angles);
    QualityReport::compute_overall(
        sharpness,
        resolution,
        pose,
        illumination,
        uniformity,
        occlusion,
        expression,
        gaze,
        pose_angles,
    )
}
