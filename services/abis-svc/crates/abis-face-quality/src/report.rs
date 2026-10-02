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

//! QualityReport: aggregate of all ISO/IEC 29794-5 metrics.

use crate::metrics::PoseAngles;
use serde::{Deserialize, Serialize};

/// Full quality report for one face image.
///
/// Each field is a normalised score in [0.0, 1.0] where 1.0 is highest quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Laplacian variance sharpness score (ISO 29794-5 clause 6.1).
    pub sharpness: f32,
    /// Interocular-distance resolution score (ISO 19794-5 clause 7.2.1).
    pub resolution: f32,
    /// Pose quality score (yaw/pitch/roll composite).
    pub pose: f32,
    /// Estimated pose angles in degrees.
    pub pose_angles: PoseAngles,
    /// Illumination quality (mean intensity + histogram spread).
    pub illumination: f32,
    /// Background uniformity score.
    pub uniformity: f32,
    /// Occlusion score (skin-tone coverage fraction).
    pub occlusion: f32,
    /// Expression neutrality score.
    pub expression: f32,
    /// Gaze quality score.
    pub gaze: f32,
    /// Composite overall quality score (weighted harmonic mean).
    pub overall: f32,
}

impl QualityReport {
    /// Compute the overall score as a weighted harmonic mean of all metrics.
    ///
    /// Weights reflect relative importance per ISO 29794-5 Annex B guidance.
    pub fn compute_overall(
        sharpness: f32,
        resolution: f32,
        pose: f32,
        illumination: f32,
        uniformity: f32,
        occlusion: f32,
        expression: f32,
        gaze: f32,
        pose_angles: PoseAngles,
    ) -> Self {
        // Weighted arithmetic mean (weights sum to 1.0).
        let weights = [
            (sharpness, 0.20f32),
            (resolution, 0.15),
            (pose, 0.20),
            (illumination, 0.15),
            (uniformity, 0.10),
            (occlusion, 0.10),
            (expression, 0.05),
            (gaze, 0.05),
        ];
        let overall: f32 = weights.iter().map(|(v, w)| v * w).sum();
        Self {
            sharpness,
            resolution,
            pose,
            pose_angles,
            illumination,
            uniformity,
            occlusion,
            expression,
            gaze,
            overall: overall.clamp(0.0, 1.0),
        }
    }

    /// Returns true when the overall score meets the minimum threshold.
    pub fn passes(&self, min_overall: f32) -> bool {
        self.overall >= min_overall
    }
}
