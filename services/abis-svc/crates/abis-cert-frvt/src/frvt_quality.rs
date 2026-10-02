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

//! FRVT Quality track submission.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrvtQualityError {
    #[error("quality score {score} out of [0, 100] range")]
    OutOfRange { score: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrvtQualityReport {
    pub mean_quality: f64,
    pub correlation_with_fmr: f64,
    pub sample_count: u64,
}

pub struct FrvtQualitySubmission;

impl FrvtQualitySubmission {
    /// Computes quality statistics from a set of quality scores.
    ///
    /// # Errors
    /// Returns an error if any score is outside [0, 100].
    pub fn run(&self, scores: &[f64]) -> Result<FrvtQualityReport, FrvtQualityError> {
        for &s in scores {
            if !(0.0..=100.0).contains(&s) {
                return Err(FrvtQualityError::OutOfRange { score: s });
            }
        }
        let mean = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
        // Stub: correlation is proportional to mean (higher quality = better match).
        let correlation = (mean / 100.0).clamp(0.0, 1.0);
        Ok(FrvtQualityReport {
            mean_quality: mean,
            correlation_with_fmr: correlation,
            sample_count: scores.len() as u64,
        })
    }
}
