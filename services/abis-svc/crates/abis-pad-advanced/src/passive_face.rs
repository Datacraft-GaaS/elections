//! Passive face PAD via texture analysis.
//!
//! Applies Local Binary Patterns (LBP), colour moments, and moire-pattern
//! detection to distinguish live faces from screen-replay attacks.

use crate::{Decision, PadError};
use serde::{Deserialize, Serialize};

/// RGB image with width x height pixels (8-bit per channel, interleaved).
#[derive(Debug, Clone)]
pub struct RgbImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Result from passive face PAD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveFaceResult {
    pub decision: Decision,
    /// LBP histogram uniformity score in [0, 1].
    pub lbp_score: f64,
    /// Colour-moment distance from live reference in [0, 1].
    pub colour_score: f64,
    /// Moire-pattern energy indicating screen-replay likelihood in [0, 1].
    pub moire_score: f64,
    /// Fused liveness score in [0, 1].
    pub fused_score: f64,
}

pub struct PassiveFaceVerifier {
    threshold: f64,
}

impl PassiveFaceVerifier {
    pub fn new(threshold: f64) -> Result<Self, PadError> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(PadError::InvalidInput {
                detail: "threshold must be in [0, 1]".into(),
            });
        }
        Ok(Self { threshold })
    }

    /// Analyses a face image for liveness.
    ///
    /// # Errors
    /// Returns `PadError::InvalidInput` if the image buffer is inconsistent.
    pub fn analyse(&self, image: &RgbImage) -> Result<PassiveFaceResult, PadError> {
        let expected = (image.width * image.height * 3) as usize;
        if image.pixels.len() != expected {
            return Err(PadError::InvalidInput {
                detail: format!(
                    "pixel buffer length {} != expected {}",
                    image.pixels.len(),
                    expected
                ),
            });
        }
        if image.pixels.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "empty image".into(),
            });
        }

        let lbp_score = lbp_uniformity(image);
        let colour_score = colour_moment_score(image);
        let moire_score = moire_energy(image);

        // Moire indicates replay: invert it for the liveness contribution.
        let fused_score =
            (lbp_score * 0.4 + colour_score * 0.3 + (1.0 - moire_score) * 0.3).clamp(0.0, 1.0);

        let decision = if fused_score >= self.threshold {
            Decision::Live
        } else {
            Decision::Spoof
        };

        Ok(PassiveFaceResult {
            decision,
            lbp_score,
            colour_score,
            moire_score,
            fused_score,
        })
    }
}

/// Computes LBP histogram uniformity over the luma channel.
fn lbp_uniformity(image: &RgbImage) -> f64 {
    // Stub: derive luma, compute simple local variance as proxy for LBP richness.
    let luma: Vec<f64> = image
        .pixels
        .chunks_exact(3)
        .map(|p| 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64)
        .collect();

    if luma.is_empty() {
        return 0.0;
    }
    let mean = luma.iter().sum::<f64>() / luma.len() as f64;
    let variance = luma.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / luma.len() as f64;
    // High variance in a natural face; screen replay tends to be over-smooth.
    (variance / (128.0 * 128.0)).min(1.0)
}

/// Colour-moment distance from a neutral live-face reference.
fn colour_moment_score(image: &RgbImage) -> f64 {
    let (mut sr, mut sg, mut sb) = (0.0f64, 0.0f64, 0.0f64);
    let n = (image.width * image.height) as f64;
    if n == 0.0 {
        return 0.0;
    }
    for chunk in image.pixels.chunks_exact(3) {
        sr += chunk[0] as f64;
        sg += chunk[1] as f64;
        sb += chunk[2] as f64;
    }
    let (mr, mg, mb) = (sr / n, sg / n, sb / n);
    // Skin-tone heuristic: live faces have r > g > b.
    if mr > mg && mg > mb {
        0.75
    } else {
        0.3
    }
}

/// Estimates moire energy in the image (frequency-domain proxy).
fn moire_energy(image: &RgbImage) -> f64 {
    // Stub: measure high-frequency horizontal energy as a proxy for moire.
    let n = image.pixels.len();
    if n < 6 {
        return 0.0;
    }
    let mut energy = 0.0f64;
    for window in image.pixels.windows(6) {
        energy += (window[0] as f64 - window[3] as f64).abs();
    }
    (energy / (n as f64 * 255.0)).clamp(0.0, 1.0)
}
