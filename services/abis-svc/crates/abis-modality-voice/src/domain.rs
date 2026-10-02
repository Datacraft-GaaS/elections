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

//! Domain types for voice biometric templates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Classification of the utterance recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UtteranceType {
    Passphrase,
    FreeSpeech,
    Digits,
    TextDependent,
    TextIndependent,
}

/// A voice biometric template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceTemplate {
    pub template_id: String,
    pub utterance_type: UtteranceType,
    pub sample_rate_hz: u32,
    pub duration_ms: u32,
    pub language: String,
    pub embedding_dim: u16,
    /// Speaker embedding vector (i-vector / x-vector).
    pub embedding: Vec<f32>,
    /// MFCC feature frames, each with 13 coefficients.
    pub mfcc_features: Vec<Vec<f32>>,
    /// Signal-to-noise ratio in dB if measured.
    pub snr_db: Option<f32>,
    pub captured_at: DateTime<Utc>,
}
