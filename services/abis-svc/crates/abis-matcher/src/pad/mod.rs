//! Presentation attack detection (PAD) interface.
//!
//! `PadDetector` is a trait that each modality's PAD module implements.
//! The production path wires a vendor SDK; the in-crate `SoftwarePad` stub
//! returns Live by default (suitable for development and test environments).

/// Presentation attack type taxonomy (ISO/IEC 30107-3 artefact-based).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackType {
    /// Printed photo (2-D print attack).
    PrintedPhoto,
    /// Video replay on a screen (display/replay attack).
    DisplayReplay,
    /// Silicone finger / face overlay.
    Silicone,
    /// Latex finger mould.
    Latex,
    /// 3-D mask (face).
    Mask,
    /// Post-mortem sample.
    Cadaver,
}

/// PAD decision for a single biometric sample.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PadResult {
    /// True if the sample is classified as live/genuine.
    pub is_live: bool,
    /// Liveness score in [0.0, 1.0] (1.0 = highly confident live).
    pub score: f32,
    /// Detected attack type, if `is_live` is false.
    pub attack_type: Option<AttackType>,
}

/// Raw biometric sample (modality-agnostic byte buffer for the PAD interface).
pub struct Sample<'a> {
    pub bytes: &'a [u8],
    /// MIME / codec hint (e.g. "image/jpeg", "application/x-wsq").
    pub media_type: &'a str,
}

/// Core PAD detector contract.
///
/// Production implementations delegate to a vendor SDK (e.g. iProov, Aware, ZoOm).
/// The in-crate `SoftwarePad` stub returns Live for all inputs.
pub trait PadDetector: Send + Sync {
    fn detect(&self, sample: &Sample<'_>) -> PadResult;
}

/// Software stub that classifies every sample as live.
///
/// **Do not use in production.** Wire a vendor SDK via `PadDetector` before deployment.
pub struct SoftwarePad;

impl PadDetector for SoftwarePad {
    fn detect(&self, _sample: &Sample<'_>) -> PadResult {
        PadResult {
            is_live: true,
            score: 1.0,
            attack_type: None,
        }
    }
}
