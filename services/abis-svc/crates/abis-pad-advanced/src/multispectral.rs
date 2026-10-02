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

//! Fingerprint multispectral PAD interface.
//!
//! Accepts multi-band (white-light, NIR, SWIR, LWIR) captures and returns
//! a per-band liveness score plus a fused attack-type classification.

use crate::{Decision, PadError};
use serde::{Deserialize, Serialize};

/// Spectral band identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpectralBand {
    WhiteLight,
    Nir,
    Swir,
    Lwir,
}

/// One captured band image represented as a raw pixel buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandCapture {
    pub band: SpectralBand,
    /// Width x height pixels, 8-bit gray.
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Multi-band fingerprint capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultispectralCapture {
    pub bands: Vec<BandCapture>,
}

/// PAI attack type for multispectral analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FingerprintAttackType {
    Silicone,
    Gelatin,
    Latex,
    Paper2d,
    Ecoflex,
    Unknown,
    None,
}

/// Result from multispectral PAD analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultispectralPadResult {
    pub decision: Decision,
    pub fused_score: f64,
    pub per_band_scores: Vec<(SpectralBand, f64)>,
    pub attack_type: FingerprintAttackType,
}

/// Analyses a multispectral capture for liveness.
///
/// # Errors
/// Returns `PadError::InvalidInput` when no bands are supplied or a band
/// pixel buffer is empty.
pub fn analyse(capture: &MultispectralCapture) -> Result<MultispectralPadResult, PadError> {
    if capture.bands.is_empty() {
        return Err(PadError::InvalidInput {
            detail: "at least one spectral band required".into(),
        });
    }

    let mut per_band_scores = Vec::with_capacity(capture.bands.len());
    for band_capture in &capture.bands {
        if band_capture.pixels.is_empty() {
            return Err(PadError::InvalidInput {
                detail: format!("band {:?} has empty pixel buffer", band_capture.band),
            });
        }
        let score = band_liveness_score(band_capture);
        per_band_scores.push((band_capture.band.clone(), score));
    }

    let fused_score = fuse_scores(&per_band_scores);
    let decision = if fused_score >= 0.5 {
        Decision::Live
    } else {
        Decision::Spoof
    };
    let attack_type = classify_attack(fused_score);

    Ok(MultispectralPadResult {
        decision,
        fused_score,
        per_band_scores,
        attack_type,
    })
}

/// Stub: computes a liveness score from a single band.
/// Real implementation feeds pixels into a band-specific CNN.
fn band_liveness_score(capture: &BandCapture) -> f64 {
    // Heuristic: NIR and SWIR bands are harder to spoof; weight accordingly.
    let mean = capture.pixels.iter().map(|&p| p as f64).sum::<f64>() / capture.pixels.len() as f64;
    // Normalise mean pixel value to [0, 1] range used as a proxy liveness score.
    let base = mean / 255.0;
    // Band-specific weight adjusting for spoof-resistance ordering.
    let weight = match capture.band {
        SpectralBand::Lwir => 0.9,
        SpectralBand::Swir => 0.85,
        SpectralBand::Nir => 0.75,
        SpectralBand::WhiteLight => 0.5,
    };
    (base * weight).clamp(0.0, 1.0)
}

/// Fuses per-band scores using a weighted average.
fn fuse_scores(scores: &[(SpectralBand, f64)]) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let sum: f64 = scores.iter().map(|(_, s)| s).sum();
    sum / scores.len() as f64
}

fn classify_attack(score: f64) -> FingerprintAttackType {
    if score >= 0.5 {
        FingerprintAttackType::None
    } else {
        FingerprintAttackType::Unknown
    }
}
