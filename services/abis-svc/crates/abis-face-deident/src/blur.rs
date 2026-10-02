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

//! Blur and pixelation baselines for face de-identification.
//!
//! Reference: McPherson, R. et al. (2016). "Defeating Image Obfuscation with
//! Deep Learning." arXiv:1609.00408. (context: blur is a weak baseline;
//! pixelation is marginally stronger)

/// Apply a box-blur (mean filter) to an RGB image (3 bytes/pixel, row-major).
///
/// `radius`: half-width of the blur kernel (radius=3 => 7x7 kernel).
pub fn blur_image(pixels: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    if pixels.is_empty() || width == 0 || height == 0 {
        return pixels.to_vec();
    }
    let w = width as usize;
    let h = height as usize;
    let r = radius as usize;
    let mut out = vec![0u8; pixels.len()];

    for y in 0..h {
        for x in 0..w {
            let mut sum = [0u64; 3];
            let mut count = 0u64;
            let y0 = y.saturating_sub(r);
            let y1 = (y + r + 1).min(h);
            let x0 = x.saturating_sub(r);
            let x1 = (x + r + 1).min(w);
            for ky in y0..y1 {
                for kx in x0..x1 {
                    let idx = (ky * w + kx) * 3;
                    if idx + 2 < pixels.len() {
                        sum[0] += pixels[idx] as u64;
                        sum[1] += pixels[idx + 1] as u64;
                        sum[2] += pixels[idx + 2] as u64;
                        count += 1;
                    }
                }
            }
            let idx = (y * w + x) * 3;
            if idx + 2 < out.len() && count > 0 {
                out[idx] = (sum[0] / count) as u8;
                out[idx + 1] = (sum[1] / count) as u8;
                out[idx + 2] = (sum[2] / count) as u8;
            }
        }
    }
    out
}

/// Pixelate an RGB image by averaging each `block_size x block_size` block.
pub fn pixelate_image(pixels: &[u8], width: u32, height: u32, block_size: u32) -> Vec<u8> {
    if pixels.is_empty() || width == 0 || height == 0 || block_size == 0 {
        return pixels.to_vec();
    }
    let w = width as usize;
    let h = height as usize;
    let bs = block_size as usize;
    let mut out = vec![0u8; pixels.len()];

    let by_count = (h + bs - 1) / bs;
    let bx_count = (w + bs - 1) / bs;

    for by in 0..by_count {
        for bx in 0..bx_count {
            let y0 = by * bs;
            let x0 = bx * bs;
            let y1 = (y0 + bs).min(h);
            let x1 = (x0 + bs).min(w);
            let mut sum = [0u64; 3];
            let mut count = 0u64;
            for y in y0..y1 {
                for x in x0..x1 {
                    let idx = (y * w + x) * 3;
                    if idx + 2 < pixels.len() {
                        sum[0] += pixels[idx] as u64;
                        sum[1] += pixels[idx + 1] as u64;
                        sum[2] += pixels[idx + 2] as u64;
                        count += 1;
                    }
                }
            }
            if count == 0 {
                continue;
            }
            let avg = [
                (sum[0] / count) as u8,
                (sum[1] / count) as u8,
                (sum[2] / count) as u8,
            ];
            for y in y0..y1 {
                for x in x0..x1 {
                    let idx = (y * w + x) * 3;
                    if idx + 2 < out.len() {
                        out[idx] = avg[0];
                        out[idx + 1] = avg[1];
                        out[idx + 2] = avg[2];
                    }
                }
            }
        }
    }
    out
}
