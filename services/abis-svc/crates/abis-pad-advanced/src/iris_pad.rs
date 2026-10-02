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

//! Iris PAD via pupil dynamics and Purkinje-reflex checks.
//!
//! Detects presentation attacks including printed images and display replays
//! by checking for expected pupil area variation and glint positions.

use crate::{Decision, PadError};
use serde::{Deserialize, Serialize};

/// Iris observation: multiple frames to capture dynamics.
#[derive(Debug, Clone)]
pub struct IrisObservation {
    /// Pupil radius in pixels for each frame.
    pub pupil_radii: Vec<f32>,
    /// Whether a Purkinje reflex (bright glint) was detected in each frame.
    pub glint_detected: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisPadResult {
    pub decision: Decision,
    pub pupil_variance: f64,
    pub glint_ratio: f64,
    pub score: f64,
}

pub struct IrisPadVerifier {
    /// Minimum pupil variance required across frames to indicate live dynamics.
    min_variance: f64,
    /// Minimum fraction of frames exhibiting a Purkinje glint.
    min_glint_ratio: f64,
}

impl IrisPadVerifier {
    pub fn new(min_variance: f64, min_glint_ratio: f64) -> Result<Self, PadError> {
        if min_variance < 0.0 {
            return Err(PadError::InvalidInput {
                detail: "min_variance must be non-negative".into(),
            });
        }
        if !(0.0..=1.0).contains(&min_glint_ratio) {
            return Err(PadError::InvalidInput {
                detail: "min_glint_ratio must be in [0, 1]".into(),
            });
        }
        Ok(Self {
            min_variance,
            min_glint_ratio,
        })
    }

    /// Analyses iris observation frames for liveness.
    ///
    /// # Errors
    /// Returns `PadError::InvalidInput` when frame counts are inconsistent.
    pub fn analyse(&self, obs: &IrisObservation) -> Result<IrisPadResult, PadError> {
        if obs.pupil_radii.len() != obs.glint_detected.len() {
            return Err(PadError::InvalidInput {
                detail: "pupil_radii and glint_detected must have equal length".into(),
            });
        }
        if obs.pupil_radii.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "at least one frame required".into(),
            });
        }

        let pupil_variance = variance_f32(&obs.pupil_radii);
        let glint_ratio = obs.glint_detected.iter().filter(|&&g| g).count() as f64
            / obs.glint_detected.len() as f64;

        let variance_ok = pupil_variance >= self.min_variance;
        let glint_ok = glint_ratio >= self.min_glint_ratio;

        let score = if variance_ok { 0.5 } else { 0.0 } + if glint_ok { 0.5 } else { 0.0 };
        let decision = if variance_ok && glint_ok {
            Decision::Live
        } else {
            Decision::Spoof
        };

        Ok(IrisPadResult {
            decision,
            pupil_variance,
            glint_ratio,
            score,
        })
    }
}

fn variance_f32(values: &[f32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().map(|&v| v as f64).sum::<f64>() / values.len() as f64;
    values
        .iter()
        .map(|&v| (v as f64 - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64
}
