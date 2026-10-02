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

//! Voice template quality assessment.

use crate::domain::VoiceTemplate;

/// Detailed quality breakdown for a voice template.
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Overall quality 0.0-100.0.
    pub overall: f32,
    /// Score derived from SNR (0.0-100.0).
    pub snr_score: f32,
    /// Score based on utterance duration (0.0-100.0).
    pub duration_score: f32,
    /// Fraction of frames with detected speech activity.
    pub speech_activity_ratio: f32,
}

/// RMS threshold below which a frame is considered silent (normalized to i16 max).
const SILENCE_RMS_THRESHOLD: f32 = 0.02;

/// Compute a quality report for a voice template.
pub fn score_quality(template: &VoiceTemplate) -> QualityReport {
    if template.duration_ms == 0 {
        return QualityReport {
            overall: 0.0,
            snr_score: 0.0,
            duration_score: 0.0,
            speech_activity_ratio: 0.0,
        };
    }

    let snr_score = snr_to_score(template.snr_db);
    let duration_score = duration_to_score(template.duration_ms);
    let speech_activity_ratio = speech_activity(&template.mfcc_features);

    let overall =
        (snr_score * 0.4 + duration_score * 0.3 + speech_activity_ratio * 100.0 * 0.3).min(100.0);

    QualityReport {
        overall,
        snr_score,
        duration_score,
        speech_activity_ratio,
    }
}

/// Map SNR in dB to a 0-100 score. Optimal range: 20-40 dB.
fn snr_to_score(snr_db: Option<f32>) -> f32 {
    match snr_db {
        None => 50.0, // unknown: neutral
        Some(snr) => snr.clamp(0.0, 40.0) / 40.0 * 100.0,
    }
}

/// Map duration to a 0-100 score. Optimal range: 2000-10000 ms.
fn duration_to_score(duration_ms: u32) -> f32 {
    let d = duration_ms as f32;
    if d < 500.0 {
        return d / 500.0 * 30.0; // very short: penalize heavily
    }
    if d < 2000.0 {
        return 30.0 + (d - 500.0) / 1500.0 * 70.0;
    }
    if d <= 10000.0 {
        return 100.0;
    }
    // Over 10 seconds: slight penalty for excessive duration.
    (100.0 - (d - 10000.0) / 1000.0 * 5.0).max(50.0)
}

/// Estimate the fraction of MFCC frames that contain active speech.
///
/// Uses the energy of the first MFCC coefficient as a proxy for frame energy.
fn speech_activity(frames: &[Vec<f32>]) -> f32 {
    if frames.is_empty() {
        return 0.0;
    }
    let active = frames
        .iter()
        .filter(|frame| {
            if frame.is_empty() {
                return false;
            }
            // c0 encodes log-energy; threshold on absolute value.
            frame[0].abs() > SILENCE_RMS_THRESHOLD * 1000.0
        })
        .count();
    active as f32 / frames.len() as f32
}
