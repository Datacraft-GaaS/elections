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
