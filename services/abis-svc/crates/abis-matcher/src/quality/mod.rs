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

//! Quality gates for per-modality biometric samples.
//!
//! Each modality's quality score (NFIQ2/ICAO/ISO 29794) is checked against
//! configurable thresholds; the gate returns a disposition for the enrolment
//! or identification workflow.

/// Quality policy thresholds (all scores normalised 0.0..1.0).
#[derive(Debug, Clone)]
pub struct QualityPolicy {
    /// Minimum NFIQ2-normalised fingerprint quality to accept.
    pub min_nfiq2: f32,
    /// Minimum ICAO face quality score to accept.
    pub min_icao_score: f32,
    /// Minimum iris quality (usable-bit fraction) to accept.
    pub min_iris_quality: f32,
    /// Minimum number of usable fingers (quality >= min_nfiq2) required.
    pub min_usable_fingers: usize,
    /// Minimum number of usable iris captures required.
    pub min_usable_iris: usize,
}

impl Default for QualityPolicy {
    fn default() -> Self {
        Self {
            min_nfiq2: 0.40,        // NFIQ2 ≥ 40/100
            min_icao_score: 0.60,   // ICAO 3/4 flags pass
            min_iris_quality: 0.60, // ≥60% usable bits
            min_usable_fingers: 6,  // 6 of 10 fingers
            min_usable_iris: 1,     // at least one iris
        }
    }
}

/// Disposition returned by the quality gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QualityDecision {
    /// All quality thresholds met; proceed with matching.
    Accept,
    /// Quality marginal; re-capture should be attempted.
    Retake,
    /// Quality persistently insufficient; escalate for manual capture (officer-assisted).
    EscalateManualCapture,
}

/// Evaluate a fingerprint quality score against policy.
pub fn gate_fingerprint(quality: f32, policy: &QualityPolicy) -> QualityDecision {
    if quality >= policy.min_nfiq2 {
        QualityDecision::Accept
    } else if quality >= policy.min_nfiq2 * 0.5 {
        QualityDecision::Retake
    } else {
        QualityDecision::EscalateManualCapture
    }
}

/// Evaluate a face quality score against ICAO policy.
pub fn gate_face(quality: f32, policy: &QualityPolicy) -> QualityDecision {
    if quality >= policy.min_icao_score {
        QualityDecision::Accept
    } else if quality >= policy.min_icao_score * 0.6 {
        QualityDecision::Retake
    } else {
        QualityDecision::EscalateManualCapture
    }
}

/// Evaluate an iris quality score against policy.
pub fn gate_iris(quality: f32, policy: &QualityPolicy) -> QualityDecision {
    if quality >= policy.min_iris_quality {
        QualityDecision::Accept
    } else if quality >= policy.min_iris_quality * 0.5 {
        QualityDecision::Retake
    } else {
        QualityDecision::EscalateManualCapture
    }
}

/// Evaluate the finger count meeting the quality threshold.
pub fn gate_finger_count(qualities: &[f32], policy: &QualityPolicy) -> QualityDecision {
    let usable = qualities.iter().filter(|&&q| q >= policy.min_nfiq2).count();
    if usable >= policy.min_usable_fingers {
        QualityDecision::Accept
    } else if usable >= policy.min_usable_fingers / 2 {
        QualityDecision::Retake
    } else {
        QualityDecision::EscalateManualCapture
    }
}
