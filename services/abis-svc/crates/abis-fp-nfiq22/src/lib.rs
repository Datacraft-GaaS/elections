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

//! NFIQ 2.2 fingerprint image quality assessment.
//!
//! Implements the 14-feature set and calibrated Random Forest classifier defined in:
//! NIST Special Publication 500-333, "NFIQ 2.2 Fingerprint Image Quality,"
//! F. Schwaiger et al., 2021.
//!
//! The quality score is a non-negative integer in [0, 100] where higher is better.
#![warn(missing_docs)]

pub mod calibrator;
pub mod features;
pub mod model;
pub mod scorer;

pub use calibrator::Calibrator;
pub use features::{extract_features, Nfiq22Features};
pub use model::RandomForestModel;
pub use scorer::nfiq22_score;

/// Gray-scale fingerprint image (8-bit pixels, row-major).
#[derive(Debug, Clone)]
pub struct GrayImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl GrayImage {
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        debug_assert_eq!(pixels.len(), (width * height) as usize);
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        self.pixels[(y * self.width + x) as usize]
    }
}

/// NFIQ 2.2 error type.
#[derive(Debug, thiserror::Error)]
pub enum Nfiq22Error {
    #[error("image too small: {0}x{1} (minimum 32x32)")]
    ImageTooSmall(u32, u32),
    #[error("model load error: {0}")]
    ModelLoad(String),
    #[error("feature extraction failed: {0}")]
    FeatureFailed(String),
}
