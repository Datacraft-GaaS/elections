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

//! Texture-based fingerprint matching.
//!
//! Implements three complementary texture descriptors:
//! - FingerCode (Jain et al., IEEE TIP 2000): filterbank + sector statistics.
//! - LBP: Local Binary Patterns per-pixel histogram.
//! - MB-LBP: Multi-Block LBP for scale invariance.
//!
//! Used as a fallback when minutiae extraction quality is low.
#![warn(missing_docs)]

pub mod fingercode;
pub mod lbp;
pub mod matcher;
pub mod mb_lbp;

pub use fingercode::{
    encode_fingercode, filter_bank, locate_reference_point, match_fingercode, tessellate,
    FilteredImage, FingerCode, Sector,
};
pub use lbp::{chi_square_distance, lbp_histogram, lbp_histogram_for_image};
pub use matcher::TextureMatcher;
pub use mb_lbp::{mb_lbp_histogram, MbLbpBlockSize};

use thiserror::Error;

/// A grayscale image with row-major pixel buffer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrayImage {
    pub width: u32,
    pub height: u32,
    /// Row-major pixels, length = width * height.
    pub pixels: Vec<u8>,
}

impl GrayImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, TextureError> {
        if pixels.len() != (width * height) as usize {
            return Err(TextureError::InvalidDimensions { width, height });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    #[inline]
    pub fn pixel(&self, x: i32, y: i32) -> u8 {
        // Replicate-border clamping prevents artificial edge gradients at
        // image boundaries that would create spurious orientation field values.
        let x = x.clamp(0, self.width as i32 - 1) as u32;
        let y = y.clamp(0, self.height as i32 - 1) as u32;
        self.pixels[(y * self.width + x) as usize]
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, v: u8) {
        self.pixels[(y * self.width + x) as usize] = v;
    }
}

/// A 2-D point with f32 coordinates.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Errors produced by this crate.
#[derive(Debug, Error)]
pub enum TextureError {
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("image too small for texture analysis (minimum 16x16)")]
    ImageTooSmall,
    #[error("could not locate a reference point in the image")]
    NoReferencePoint,
    #[error("inconsistent feature vector length: expected {expected}, got {got}")]
    FeatureLengthMismatch { expected: usize, got: usize },
}
