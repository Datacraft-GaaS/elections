//! LivDet-family fingerprint PAD facade.
//!
//! Defines a pluggable `Detector` trait, a `LivDetVerifier` that dispatches
//! to registered detectors, and a baseline heuristic using ridge-frequency
//! and valley-fill analysis.

use crate::{Decision, PadError};
use serde::{Deserialize, Serialize};

/// Gray-scale fingerprint image (8-bit pixels).
#[derive(Debug, Clone)]
pub struct GrayImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// PAI attack medium classification per LivDet categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivDetAttack {
    Ecoflex,
    Gelatin,
    Latex,
    WoodGlue,
    Silgum,
    Unknown,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivDetResult {
    pub decision: Decision,
    pub score: f64,
    pub attack: LivDetAttack,
    pub detector_id: String,
}

/// Pluggable detector trait for LivDet-family algorithms.
pub trait Detector: Send + Sync {
    fn id(&self) -> &str;
    fn score(&self, image: &GrayImage) -> Result<f64, PadError>;
}

/// Baseline heuristic detector: ridge-frequency + valley-fill analysis.
pub struct BaselineDetector;

impl Detector for BaselineDetector {
    fn id(&self) -> &str {
        "baseline-ridge-freq"
    }

    fn score(&self, image: &GrayImage) -> Result<f64, PadError> {
        if image.pixels.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "empty fingerprint image".into(),
            });
        }
        Ok(ridge_frequency_score(image) * 0.6 + valley_fill_score(image) * 0.4)
    }
}

pub struct LivDetVerifier {
    detectors: Vec<Box<dyn Detector>>,
    threshold: f64,
}

impl LivDetVerifier {
    pub fn new(threshold: f64) -> Result<Self, PadError> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(PadError::InvalidInput {
                detail: "threshold must be in [0, 1]".into(),
            });
        }
        Ok(Self {
            detectors: vec![Box::new(BaselineDetector)],
            threshold,
        })
    }

    pub fn register(&mut self, detector: Box<dyn Detector>) {
        self.detectors.push(detector);
    }

    /// Runs all registered detectors and fuses scores by averaging.
    ///
    /// # Errors
    /// Propagates errors from detectors.
    pub fn analyse(&self, image: &GrayImage) -> Result<LivDetResult, PadError> {
        if self.detectors.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "no detectors registered".into(),
            });
        }
        let mut total = 0.0f64;
        let mut best_id = String::new();
        for det in &self.detectors {
            let s = det.score(image)?;
            total += s;
            if best_id.is_empty() {
                best_id = det.id().to_owned();
            }
        }
        let score = total / self.detectors.len() as f64;
        let decision = if score >= self.threshold {
            Decision::Live
        } else {
            Decision::Spoof
        };
        let attack = if score < self.threshold {
            LivDetAttack::Unknown
        } else {
            LivDetAttack::None
        };
        Ok(LivDetResult {
            decision,
            score,
            attack,
            detector_id: best_id,
        })
    }
}

/// Estimates ridge frequency regularity as a liveness proxy.
fn ridge_frequency_score(image: &GrayImage) -> f64 {
    let n = image.pixels.len();
    if n < 2 {
        return 0.0;
    }
    let transitions: usize = image
        .pixels
        .windows(2)
        .filter(|w| (w[0] as i16 - w[1] as i16).unsigned_abs() > 64)
        .count();
    // Live fingers exhibit regular ridge transitions; spoofs are smoother.
    let rate = transitions as f64 / (n - 1) as f64;
    rate.clamp(0.0, 1.0)
}

/// Estimates valley-fill degree: spoofs tend to have filled valleys.
fn valley_fill_score(image: &GrayImage) -> f64 {
    if image.pixels.is_empty() {
        return 0.0;
    }
    let dark_pixels = image.pixels.iter().filter(|&&p| p < 80).count();
    let ratio = dark_pixels as f64 / image.pixels.len() as f64;
    // Live: ~30-50% dark (valleys); spoofs have fewer dark pixels.
    if (0.25..0.55).contains(&ratio) {
        0.8
    } else {
        0.3
    }
}
