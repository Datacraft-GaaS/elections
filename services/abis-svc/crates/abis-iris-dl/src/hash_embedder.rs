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

//! Hash-based deterministic iris embedder.
//!
//! Produces a 256-dimensional unit-normalised vector by hashing image pixels
//! with SHA-256 and 8 distinct salts, then L2-normalising the result.
//! Serves as a build-without-ONNX fallback for the DeepIrisNet embedder.

use eel_abis_iris_segment::GrayImage;
use sha2::{Digest, Sha256};

use crate::embedder::Embedding;

/// Number of output dimensions.
const EMBED_DIM: usize = 256;
/// Number of hash salts (each produces 32 bytes = 32 dimensions).
const NUM_SALTS: usize = 8;

/// Salts used to derive independent hash outputs.
const SALTS: [[u8; 4]; NUM_SALTS] = [
    [0x00, 0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00, 0x01],
    [0x00, 0x00, 0x00, 0x02],
    [0x00, 0x00, 0x00, 0x03],
    [0x00, 0x00, 0x00, 0x04],
    [0x00, 0x00, 0x00, 0x05],
    [0x00, 0x00, 0x00, 0x06],
    [0x00, 0x00, 0x00, 0x07],
];

/// Deterministic hash-based iris embedder (no model required).
pub struct HashEmbedder;

impl HashEmbedder {
    /// Hash-expand image pixels to a 256-d L2-normalised vector.
    ///
    /// The result is always unit-norm unless the image is all-zero
    /// (in which case the hash output is still non-zero, but callers
    /// should gate on image content before embedding).
    pub fn embed(image: &GrayImage) -> Embedding {
        let pixels = image.pixels();

        let mut raw = Vec::with_capacity(EMBED_DIM);
        for salt in &SALTS {
            let mut hasher = Sha256::new();
            hasher.update(salt);
            hasher.update(pixels);
            let digest = hasher.finalize();
            // Each SHA-256 output is 32 bytes; map each byte to a signed float in (-1, 1).
            for byte in digest.iter() {
                raw.push(*byte as f32 - 127.5);
            }
        }

        debug_assert_eq!(raw.len(), EMBED_DIM);
        l2_normalise(raw)
    }
}

fn l2_normalise(mut v: Vec<f32>) -> Embedding {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > f32::EPSILON {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    Embedding { vector: v }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(w: u32, h: u32, fill: u8) -> GrayImage {
        GrayImage::from_raw(w, h, vec![fill; (w * h) as usize])
    }

    #[test]
    fn embed_produces_256_dimensions() {
        let img = make_image(8, 8, 42);
        let e = HashEmbedder::embed(&img);
        assert_eq!(e.vector.len(), 256);
    }

    #[test]
    fn embed_is_unit_normalised() {
        let img = make_image(8, 8, 99);
        let e = HashEmbedder::embed(&img);
        let norm: f32 = e.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "norm: {norm}");
    }

    #[test]
    fn embed_deterministic() {
        let img1 = make_image(4, 4, 55);
        let img2 = make_image(4, 4, 55);
        let e1 = HashEmbedder::embed(&img1);
        let e2 = HashEmbedder::embed(&img2);
        assert_eq!(e1.vector, e2.vector);
    }
}
