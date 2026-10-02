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

//! Bilinear warp: applies a SimilarityTransform to produce a 112x112 crop.
//!
//! For each destination pixel (dx, dy) the inverse transform gives the source
//! (sx, sy); pixel value is bilinear-interpolated from the source image.
//!
//! Reference: Gonzalez & Woods (2018). Digital Image Processing, 4th ed.
//! Chapter 2.6 "Geometric Transformations".

use crate::transform::{AlignError, SimilarityTransform};
use crate::ALIGN_SIZE;

/// Aligned face image: ALIGN_SIZE x ALIGN_SIZE, RGB, 3 bytes per pixel.
#[derive(Debug, Clone)]
pub struct AlignedFace {
    pub pixels: Vec<u8>,
    pub size: u32,
}

impl AlignedFace {
    pub fn pixel(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let idx = ((y * self.size + x) * 3) as usize;
        if idx + 2 < self.pixels.len() {
            (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2])
        } else {
            (0, 0, 0)
        }
    }
}

/// Bilinear interpolation at fractional position (sx, sy) in `src`.
fn bilinear(src: &[u8], src_w: u32, src_h: u32, sx: f32, sy: f32) -> [u8; 3] {
    let x0 = sx.floor() as i32;
    let y0 = sy.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let fx = sx - sx.floor();
    let fy = sy - sy.floor();

    let sample = |xi: i32, yi: i32| -> [f32; 3] {
        if xi < 0 || yi < 0 || xi >= src_w as i32 || yi >= src_h as i32 {
            return [0.0, 0.0, 0.0];
        }
        let idx = ((yi as u32 * src_w + xi as u32) * 3) as usize;
        if idx + 2 < src.len() {
            [src[idx] as f32, src[idx + 1] as f32, src[idx + 2] as f32]
        } else {
            [0.0, 0.0, 0.0]
        }
    };

    let p00 = sample(x0, y0);
    let p10 = sample(x1, y0);
    let p01 = sample(x0, y1);
    let p11 = sample(x1, y1);

    let mut result = [0u8; 3];
    for ch in 0..3 {
        let v = p00[ch] * (1.0 - fx) * (1.0 - fy)
            + p10[ch] * fx * (1.0 - fy)
            + p01[ch] * (1.0 - fx) * fy
            + p11[ch] * fx * fy;
        result[ch] = v.round().clamp(0.0, 255.0) as u8;
    }
    result
}

/// Warp `src` (RGB, `src_w x src_h`) using the similarity transform `tf`,
/// producing an `ALIGN_SIZE x ALIGN_SIZE` aligned face.
///
/// Uses inverse mapping: for each destination pixel compute the source
/// coordinate via `tf.inverse()`, then bilinear-interpolate.
pub fn warp_bilinear(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    tf: &SimilarityTransform,
) -> Result<AlignedFace, AlignError> {
    let inv = tf.inverse()?;
    let sz = ALIGN_SIZE;
    let mut out = vec![0u8; (sz * sz * 3) as usize];

    for dy in 0..sz {
        for dx in 0..sz {
            let (sx, sy) = inv.apply_point(dx as f32, dy as f32);
            let rgb = bilinear(src, src_w, src_h, sx, sy);
            let idx = ((dy * sz + dx) * 3) as usize;
            out[idx] = rgb[0];
            out[idx + 1] = rgb[1];
            out[idx + 2] = rgb[2];
        }
    }

    Ok(AlignedFace {
        pixels: out,
        size: sz,
    })
}
