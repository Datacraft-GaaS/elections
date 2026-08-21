//! Matcher trait and per-modality implementations.

pub mod face_matcher;
pub mod fingerprint_matcher;
pub mod fusion;
pub mod iris_matcher;

pub use face_matcher::FaceMatcher;
pub use fingerprint_matcher::FingerprintMatcher;
pub use fusion::{FusionPolicy, FusionResult, MultiModalScore};
pub use iris_matcher::IrisMatcher;

use crate::modality::Template;

/// Comparison score returned by every matcher.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Score {
    /// Raw algorithm-native score (e.g. minutiae count, cosine similarity, 1-HD).
    pub raw: f32,
    /// Score normalised to [0.0, 1.0] by the matcher's decision function.
    pub normalised: f32,
    /// Whether `normalised` exceeds the configured decision threshold.
    pub threshold_pass: bool,
}

impl Score {
    pub fn new(raw: f32, normalised: f32, threshold: f32) -> Self {
        Self {
            raw,
            normalised,
            threshold_pass: normalised >= threshold,
        }
    }
}

/// Core matching contract.
pub trait Matcher<T: Template>: Send + Sync {
    /// Compare probe to gallery template and return a composite score.
    fn compare(&self, probe: &T, gallery: &T) -> Score;
}
