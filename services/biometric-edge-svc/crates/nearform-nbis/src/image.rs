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

//! FingerprintImage — wraps raw grayscale PGM/PPM pixels for the
//! NBIS extraction pipeline. Avoids dependencies on `image` crate to keep
//! the build simple and consistent with NBIS's own PGM reader.

use super::NearformError;
use std::path::Path;

/// Raw fingerprint image: 8-bit grayscale, rectangular.
#[derive(Debug, Clone)]
pub struct FingerprintImage {
    pub width: u32,
    pub height: u32,
    /// Row-major pixel data, length = width * height.
    pub pixels: Vec<u8>,
}

impl FingerprintImage {
    pub fn from_grayscale(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, NearformError> {
        let expected = (width as usize) * (height as usize);
        if pixels.len() != expected {
            return Err(NearformError::InvalidDimensions {
                bytes: pixels.len(),
                w: width,
                h: height,
            });
        }
        Ok(Self { width, height, pixels })
    }

    pub fn from_pgm_bytes(bytes: &[u8], width: u32, height: u32) -> Result<Self, NearformError> {
        Self::from_grayscale(width, height, bytes.to_vec())
    }

    pub fn from_pgm_or_pbm_bytes(
        bytes: &[u8], width: u32, height: u32,
    ) -> Result<Self, NearformError> {
        Self::from_grayscale(width, height, bytes.to_vec())
    }

    /// Load a PGM file from disk (handles P2 ASCII and P5 binary formats).
    pub fn load_from_file(path: &Path) -> Result<Self, NearformError> {
        let data = std::fs::read(path)?;
        if data.len() < 10 {
            return Err(NearformError::Decode("file too small".into()));
        }
        if &data[0..2] != b"P2" && &data[0..2] != b"P5" {
            return Err(NearformError::UnsupportedFormat(
                "expected PGM (P2/P5)".into(),
            ));
        }
        // PGM header: magic, width height, maxval, then raster.
        // Skip comments and whitespace.
        let mut pos = 2;
        let mut tokens = Vec::new();
        while tokens.len() < 4 && pos < data.len() {
            while pos < data.len() && (data[pos] as char).is_ascii_whitespace() {
                pos += 1;
            }
            if pos < data.len() && data[pos] == b'#' {
                while pos < data.len() && data[pos] != b'\n' {
                    pos += 1;
                }
                continue;
            }
            let start = pos;
            while pos < data.len() && !(data[pos] as char).is_ascii_whitespace() {
                pos += 1;
            }
            if pos > start {
                tokens.push(std::str::from_utf8(&data[start..pos]).unwrap_or("0"));
            }
        }
        if tokens.len() < 3 {
            return Err(NearformError::Decode("PGM header incomplete".into()));
        }
        let width: u32 = tokens[0].parse().map_err(|_| NearformError::Decode("bad width".into()))?;
        let height: u32 = tokens[1].parse().map_err(|_| NearformError::Decode("bad height".into()))?;
        let _maxval: u32 = tokens[2].parse().map_err(|_| NearformError::Decode("bad maxval".into()))?;

        // Skip to after maxval newline
        while pos < data.len() && (data[pos] as char).is_ascii_whitespace() {
            pos += 1;
        }
        let pixels = data[pos..].to_vec();
        Self::from_pgm_bytes(&pixels, width, height)
    }

    pub fn row(&self, y: u32) -> &[u8] {
        let y = y as usize;
        let w = self.width as usize;
        &self.pixels[y * w..(y + 1) * w]
    }

    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        let x = x as usize;
        let y = y as usize;
        self.pixels[y * (self.width as usize) + x]
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, v: u8) {
        let x = x as usize;
        let y = y as usize;
        self.pixels[y * (self.width as usize) + x] = v;
    }
}
