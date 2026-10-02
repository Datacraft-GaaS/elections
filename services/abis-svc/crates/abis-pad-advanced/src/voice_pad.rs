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

//! Voice PAD.
//!
//! Detects replay (magnitude-spectrum modulation), synthesis (constant F0 jitter),
//! and voice conversion (formant discontinuity).

use crate::{Decision, PadError};
use serde::{Deserialize, Serialize};

/// Voice attack category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceAttack {
    Replay,
    TtsSynthesis,
    VoiceConversion,
    None,
}

/// Input voice features extracted from an audio frame.
#[derive(Debug, Clone)]
pub struct VoiceFeatures {
    /// Short-time Fourier transform magnitude spectrum.
    pub magnitude_spectrum: Vec<f32>,
    /// Per-frame fundamental frequency (F0) in Hz; empty means unvoiced.
    pub f0_track: Vec<f32>,
    /// Formant frequencies F1..F4 per frame.
    pub formants: Vec<[f32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePadResult {
    pub decision: Decision,
    pub attack: VoiceAttack,
    pub replay_score: f64,
    pub synthesis_score: f64,
    pub conversion_score: f64,
}

pub struct VoicePadVerifier {
    threshold: f64,
}

impl VoicePadVerifier {
    pub fn new(threshold: f64) -> Result<Self, PadError> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(PadError::InvalidInput {
                detail: "threshold must be in [0, 1]".into(),
            });
        }
        Ok(Self { threshold })
    }

    /// Analyses voice features for PAD.
    ///
    /// # Errors
    /// Returns `PadError::InvalidInput` if the feature buffers are empty.
    pub fn analyse(&self, features: &VoiceFeatures) -> Result<VoicePadResult, PadError> {
        if features.magnitude_spectrum.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "magnitude_spectrum must not be empty".into(),
            });
        }

        let replay_score = replay_modulation_score(&features.magnitude_spectrum);
        let synthesis_score = if features.f0_track.is_empty() {
            0.0
        } else {
            constant_f0_jitter_score(&features.f0_track)
        };
        let conversion_score = if features.formants.is_empty() {
            0.0
        } else {
            formant_discontinuity_score(&features.formants)
        };

        // Spoof scores are high; live scores are low.
        let max_spoof = replay_score.max(synthesis_score).max(conversion_score);
        let liveness = (1.0 - max_spoof).clamp(0.0, 1.0);

        let (decision, attack) = if liveness >= self.threshold {
            (Decision::Live, VoiceAttack::None)
        } else if replay_score >= synthesis_score && replay_score >= conversion_score {
            (Decision::Spoof, VoiceAttack::Replay)
        } else if synthesis_score >= conversion_score {
            (Decision::Spoof, VoiceAttack::TtsSynthesis)
        } else {
            (Decision::Spoof, VoiceAttack::VoiceConversion)
        };

        Ok(VoicePadResult {
            decision,
            attack,
            replay_score,
            synthesis_score,
            conversion_score,
        })
    }
}

/// Replay attack: characteristic modulation sidebands in the magnitude spectrum.
fn replay_modulation_score(spectrum: &[f32]) -> f64 {
    // Heuristic: replay introduces periodic peaks every ~N bins.
    let n = spectrum.len();
    if n < 4 {
        return 0.0;
    }
    let mut sideband_energy = 0.0f64;
    let step = (n / 8).max(1);
    for i in (step..n).step_by(step) {
        sideband_energy += spectrum[i] as f64;
    }
    let total: f64 = spectrum.iter().map(|&v| v as f64).sum();
    if total == 0.0 {
        return 0.0;
    }
    (sideband_energy / total).clamp(0.0, 1.0)
}

/// TTS synthesis: excessively constant F0 jitter (too smooth).
fn constant_f0_jitter_score(f0_track: &[f32]) -> f64 {
    if f0_track.len() < 2 {
        return 0.0;
    }
    let jitter: Vec<f64> = f0_track
        .windows(2)
        .map(|w| (w[1] as f64 - w[0] as f64).abs())
        .collect();
    let mean_jitter = jitter.iter().sum::<f64>() / jitter.len() as f64;
    // Very low jitter means synthesis; natural speech has >0.5 Hz variation.
    if mean_jitter < 0.5 {
        0.85
    } else {
        0.1
    }
}

/// Voice conversion: formant discontinuities at frame boundaries.
fn formant_discontinuity_score(formants: &[[f32; 4]]) -> f64 {
    if formants.len() < 2 {
        return 0.0;
    }
    let mut total_disc = 0.0f64;
    for pair in formants.windows(2) {
        let disc: f64 = pair[0]
            .iter()
            .zip(pair[1].iter())
            .map(|(&a, &b)| (a as f64 - b as f64).abs())
            .sum();
        total_disc += disc;
    }
    let mean_disc = total_disc / ((formants.len() - 1) * 4) as f64;
    // High mean discontinuity indicates voice conversion.
    (mean_disc / 500.0).clamp(0.0, 1.0)
}
