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

pub mod image;
pub mod extraction;
pub mod template;
pub mod bozorth;
pub mod gallery;
pub mod nfiq;
pub mod ffi;
pub mod operator_selection;

pub use extraction::{FingerprintExtractor, ExtractionResult};
pub use template::{Minutia, MinutiaType, Template};
pub use bozorth::{Bozorth3, MatchResult};
pub use gallery::{Gallery, CandidateMatch};
pub use nfiq::{NfiqResult, assess_quality};

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
        Bozorth3::default().match_templates(probe, enrolled, threshold)
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
    pub fn quality(image_bytes: &[u8], width: u32, height: u32) -> Result<NfiqResult, NearformError> {
        let img = image::FingerprintImage::from_pgm_or_pbm_bytes(image_bytes, width, height)?;
        Ok(assess_quality(&img))
    }

    /// Compatibility with the old `BiometricCapture.extract(&Path)` API.
    /// Loads a PGM/PPM/PBM/BMP from disk.
    pub fn extract_from_file(path: &std::path::Path) -> Result<ExtractionResult, NearformError> {
        let img = image::FingerprintImage::load_from_file(path)?;
        let extractor = FingerprintExtractor::default();
        let template = extractor.extract(&img)?;
        Ok(ExtractionResult {
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
