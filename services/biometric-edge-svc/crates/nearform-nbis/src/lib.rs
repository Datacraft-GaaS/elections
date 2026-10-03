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

//! `nearform-nbis` — NIST NBIS-compatible fingerprint extraction & matching.
//!
//! Clean-room Rust reimplementation of the NIST-Developed Bozoroth3 and
//! Bazen & Gerez minutiae algorithms. Pure Rust, no FFI, no proprietary deps.
//!
//! ## Algorithms
//! - **Extraction** (Bazen & Gerez / NIST NBIS mindtct-style):
//!   1. Image normalization (mean/variance equalization)
//!   2. Orientation field (gradient-based)
//!   3. Ridge frequency (local frequency analysis)
//!   4. Binarization via thresholded orientation-aware filter
//!   5. Thinning (Zhang-Suen skeletonization)
//!   6. Minutiae detection: ridge endings + bifurcations
//!   7. Quality scoring (ridge clarity + minutiae count)
//!
//! - **Matching** (Bozorth3, NIST-Developed, 2003):
//!   1. Build minutiae compatibility tables
//!   2. Pair compatible minutiae (Galton-edge)
//!   3. Chain propagation forward + backward
//!   4. Score by sum of inter-minutiae distances
//!
//! ## Quality
//! Tunable thresholds mapping to operational FAR/FRR:
//! - threshold 25 → permissive (FAR ~1%)
//! - threshold 40 → NIST standard (FAR ~0.1%)
//! - threshold 50 → strict (FAR ~0.01%)
//! - threshold 70 → very strict (FAR ~0.001%)

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod bozorth;
pub mod extraction;
pub mod ffi;
pub mod gallery;
pub mod image;
pub mod mindtct_ffi;
pub mod nfiq;
pub mod operator_selection;
pub mod template;

pub use bozorth::Bozorth3;
pub use extraction::{ExtractionResult as PureExtractionResult, FingerprintExtractor};
pub use gallery::{CandidateMatch, Gallery};
pub use nfiq::{assess_quality, NfiqResult};
pub use template::{Minutia, MinutiaType, Template};

/// High-level pipeline: image → template → match.
pub struct FingerprintEngine;

impl FingerprintEngine {
    /// Extract a fingerprint template from a grayscale image.
    pub fn extract(image_bytes: &[u8], width: u32, height: u32) -> Result<Template, NearformError> {
        let img = image::FingerprintImage::from_pgm_or_pbm_bytes(image_bytes, width, height)?;
        let extractor = FingerprintExtractor::default();
        extractor.extract(&img)
    }

    /// 1:1 match between two templates.
    pub fn match_1_to_1(probe: &Template, enrolled: &Template, threshold: u32) -> MatchResult {
        Bozorth3.match_templates(probe, enrolled, threshold)
    }

    /// 1:N search against a gallery.
    pub fn match_1_to_n(
        probe: &Template,
        gallery: &Gallery,
        threshold: u32,
    ) -> Vec<CandidateMatch> {
        gallery.search(probe, threshold)
    }

    /// Assess image quality (NFIQ-style: 1=excellent, 5=poor).
    ///
    /// Extracts minutiae with the pure-Rust extractor, then maps the
    /// minutiae set onto the 1–5 scale (see [`nfiq`] for the mapping and
    /// why the C classifier is not wired).
    pub fn quality(
        image_bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Result<NfiqResult, NearformError> {
        let img = image::FingerprintImage::from_pgm_or_pbm_bytes(image_bytes, width, height)?;
        let template = FingerprintExtractor::default().extract(&img)?;
        Ok(assess_quality(&template))
    }

    /// Compatibility with the old `BiometricCapture.extract(&Path)` API.
    /// Loads a PGM/PPM/PBM/BMP from disk.
    pub fn extract_from_file(
        path: &std::path::Path,
    ) -> Result<PureExtractionResult, NearformError> {
        let img = image::FingerprintImage::load_from_file(path)?;
        let extractor = FingerprintExtractor::default();
        let template = extractor.extract(&img)?;
        Ok(PureExtractionResult {
            template,
            image_path: path.to_path_buf(),
        })
    }
}

#[derive(Debug, Error)]
pub enum NearformError {
    #[error("image dimensions invalid: {w}x{h} for {bytes} bytes")]
    InvalidDimensions { bytes: usize, w: u32, h: u32 },

    #[error("image decode failed: {0}")]
    Decode(String),

    #[error("extraction failed: {0}")]
    Extraction(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Errors from the NBIS binary-driven flows (shell-out layer in
/// [`bozorth`] and [`mindtct`]).
#[derive(Debug, Error)]
pub enum NbError {
    #[error("NBIS binary not found: {0}")]
    BinaryNotFound(String),
    #[error("NBIS tool {tool} failed (exit {exit_code}): {stderr}")]
    ToolFailed {
        tool: String,
        exit_code: i32,
        stderr: String,
    },
    #[error("io: {0}")]
    Io(String),
    #[error("os io: {0}")]
    Os(#[from] std::io::Error),
    #[error("template not found: {0}")]
    TemplateNotFound(String),
}

/// Match verdict shared by the FFI matcher and the binary-driven flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    /// Bozorth3 integer score (higher = better match).
    pub score: u32,
    /// Threshold applied.
    pub threshold: u32,
    /// score >= threshold.
    pub matched: bool,
}

/// Extraction output mode for the binary-driven mindtct flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionMode {
    /// `.xyt` minutiae text output.
    NbisXyt,
}

/// Per-template minutiae quality summary from `.xyt` output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Mean minutiae quality 0..100.
    pub mean_minutiae_quality: u32,
    /// Minimum minutiae quality 0..100.
    pub min_minutiae_quality: u32,
    /// Maximum minutiae quality 0..100.
    pub max_minutiae_quality: u32,
}

/// Extraction result of the binary-driven mindtct flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Path of the produced `.xyt` template file.
    pub template_path: std::path::PathBuf,
    /// Output mode used.
    pub mode: ExtractionMode,
    /// Minutiae lines parsed.
    pub minutiae_count: usize,
    /// Quality summary.
    pub quality: QualityMetrics,
}
