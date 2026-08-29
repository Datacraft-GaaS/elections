//! Multi-finger slap segmentation and fusion.
//!
//! Handles 4-4-2 slap acquisition:
//!   - Right four (index, middle, ring, little)
//!   - Left four (index, middle, ring, little)
//!   - Thumbs (right, left)
//!
//! Reference: ISO/IEC 29109-2:2018 fingerprint slap segmentation conformance testing.
#![warn(missing_docs)]

pub mod dpr;
pub mod matcher;
pub mod multifinger;
pub mod slap;

pub use dpr::{DprPolicy, DprResult};
pub use matcher::MultiFingerMatcher;
pub use multifinger::{MultiFingerTemplate, PerFingerScore};
pub use slap::{segment_slap, BoundingBox, FingerPosition, SlapKind};

use thiserror::Error;

/// A grayscale image (row-major).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrayImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl GrayImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, MultiFingerError> {
        if pixels.len() != (width * height) as usize {
            return Err(MultiFingerError::InvalidDimensions { width, height });
        }
        Ok(Self { width, height, pixels })
    }

    pub fn pixel(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0;
        }
        self.pixels[(y as u32 * self.width + x as u32) as usize]
    }

    /// Crop a sub-image.
    pub fn crop(&self, bb: &BoundingBox) -> GrayImage {
        let x0 = bb.x.min(self.width.saturating_sub(1));
        let y0 = bb.y.min(self.height.saturating_sub(1));
        let x1 = (bb.x + bb.width).min(self.width);
        let y1 = (bb.y + bb.height).min(self.height);
        let ow = x1 - x0;
        let oh = y1 - y0;
        let mut pixels = vec![0u8; (ow * oh) as usize];
        for dy in 0..oh {
            for dx in 0..ow {
                pixels[(dy * ow + dx) as usize] = self.pixels[((y0 + dy) * self.width + x0 + dx) as usize];
            }
        }
        GrayImage { width: ow, height: oh, pixels }
    }
}

/// Errors from this crate.
#[derive(Debug, Error)]
pub enum MultiFingerError {
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("slap segmentation failed: {0}")]
    SegmentationFailed(String),
    #[error("template finger count mismatch: probe={probe}, gallery={gallery}")]
    FingerCountMismatch { probe: usize, gallery: usize },
    #[error("DPR gate triggered: only {matched} of {available} fingers matched")]
    DprRejected { matched: usize, available: usize },
}
