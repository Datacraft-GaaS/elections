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

//! Fingerprint orientation field analysis and matching.
//!
//! Provides:
//! - Dense orientation field from image gradients (Rao 1990).
//! - Singular point detection via Poincare index (Kass & Witkin 1987, applied to fingerprints by
//!   Kawagoe & Tojo 1984).
//! - Pattern classification {Arch, TentedArch, LeftLoop, RightLoop, Whorl, TwinLoop}.
//! - Multi-granularity matcher (pattern class -> singular points -> field correlation).
#![warn(missing_docs)]

pub mod field;
pub mod matcher;
pub mod pattern;
pub mod singular;

pub use field::OrientationField;
pub use matcher::OrientationMatcher;
pub use pattern::{classify_pattern, FingerprintPattern};
pub use singular::{SingularKind, SingularPoint};

use thiserror::Error;

/// A grayscale image (row-major).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrayImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl GrayImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, OrientationError> {
        if pixels.len() != (width * height) as usize {
            return Err(OrientationError::InvalidDimensions { width, height });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    #[inline]
    pub fn pixel(&self, x: i32, y: i32) -> u8 {
        // Clamp to the nearest in-bounds pixel (replicate-border padding).
        // This prevents artificial edges at the image boundary that would
        // create spurious gradients when computing orientation fields.
        let x = x.clamp(0, self.width as i32 - 1) as u32;
        let y = y.clamp(0, self.height as i32 - 1) as u32;
        self.pixels[(y * self.width + x) as usize]
    }
}

/// A 2-D integer point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IPoint {
    pub x: i32,
    pub y: i32,
}

/// Errors produced by this crate.
#[derive(Debug, Error)]
pub enum OrientationError {
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("image too small for orientation analysis (minimum 16x16)")]
    ImageTooSmall,
    #[error("orientation field dimensions mismatch")]
    FieldMismatch,
}
