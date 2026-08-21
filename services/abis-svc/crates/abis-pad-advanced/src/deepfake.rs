//! Deepfake detector interface for face and voice modalities.
//!
//! Production integrations wire to MesoNet/FaceForensics++/RawNet2 via
//! `VendorAdapter`. This crate provides the trait boundary and a stub impl.

use crate::{Decision, PadError};
use serde::{Deserialize, Serialize};

/// Deepfake attack category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeepfakeAttackType {
    /// Face-swap or reenactment (e.g. FaceForensics++ categories).
    FaceManipulation,
    /// End-to-end GAN-generated face.
    GanFace,
    /// Neural voice cloning / text-to-speech.
    VoiceClone,
    /// No deepfake detected.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepfakeResult {
    pub decision: Decision,
    pub confidence: f64,
    pub attack_type: DeepfakeAttackType,
    pub vendor_id: String,
}

/// Pluggable vendor adapter for deepfake detection.
pub trait VendorAdapter: Send + Sync {
    fn vendor_id(&self) -> &str;
    fn detect_face(&self, embedding: &[f32]) -> Result<(f64, DeepfakeAttackType), PadError>;
    fn detect_voice(&self, features: &[f32]) -> Result<(f64, DeepfakeAttackType), PadError>;
}

/// High-level deepfake detector that dispatches to a vendor adapter.
pub struct DeepfakeDetector {
    adapter: Box<dyn VendorAdapter>,
    threshold: f64,
}

impl DeepfakeDetector {
    pub fn new(adapter: Box<dyn VendorAdapter>, threshold: f64) -> Result<Self, PadError> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(PadError::InvalidInput {
                detail: "threshold must be in [0, 1]".into(),
            });
        }
        Ok(Self { adapter, threshold })
    }

    /// Detects deepfake manipulation in a face embedding vector.
    ///
    /// # Errors
    /// Propagates errors from the vendor adapter.
    pub fn detect_face(&self, embedding: &[f32]) -> Result<DeepfakeResult, PadError> {
        if embedding.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "face embedding must not be empty".into(),
            });
        }
        let (confidence, attack_type) = self.adapter.detect_face(embedding)?;
        let decision = if confidence < self.threshold {
            Decision::Live
        } else {
            Decision::Spoof
        };
        Ok(DeepfakeResult {
            decision,
            confidence,
            attack_type,
            vendor_id: self.adapter.vendor_id().to_owned(),
        })
    }

    /// Detects deepfake manipulation in voice features.
    ///
    /// # Errors
    /// Propagates errors from the vendor adapter.
    pub fn detect_voice(&self, features: &[f32]) -> Result<DeepfakeResult, PadError> {
        if features.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "voice features must not be empty".into(),
            });
        }
        let (confidence, attack_type) = self.adapter.detect_voice(features)?;
        let decision = if confidence < self.threshold {
            Decision::Live
        } else {
            Decision::Spoof
        };
        Ok(DeepfakeResult {
            decision,
            confidence,
            attack_type,
            vendor_id: self.adapter.vendor_id().to_owned(),
        })
    }
}

/// Stub adapter returning heuristic scores from embedding statistics.
pub struct StubAdapter;

impl VendorAdapter for StubAdapter {
    fn vendor_id(&self) -> &str {
        "stub-v0"
    }

    fn detect_face(&self, embedding: &[f32]) -> Result<(f64, DeepfakeAttackType), PadError> {
        if embedding.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "empty embedding".into(),
            });
        }
        let norm: f64 = embedding
            .iter()
            .map(|&v| (v as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        // Unnormalised embeddings are suspicious.
        let confidence = if (norm - 1.0).abs() > 0.5 { 0.7 } else { 0.1 };
        let attack = if confidence > 0.5 {
            DeepfakeAttackType::FaceManipulation
        } else {
            DeepfakeAttackType::None
        };
        Ok((confidence, attack))
    }

    fn detect_voice(&self, features: &[f32]) -> Result<(f64, DeepfakeAttackType), PadError> {
        if features.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "empty features".into(),
            });
        }
        let energy: f64 = features.iter().map(|&v| v as f64).sum();
        let confidence = if energy == 0.0 { 0.9 } else { 0.1 };
        let attack = if confidence > 0.5 {
            DeepfakeAttackType::VoiceClone
        } else {
            DeepfakeAttackType::None
        };
        Ok((confidence, attack))
    }
}
