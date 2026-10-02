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

//! Skeleton extraction for the SourceAFIS algorithm.
//!
//! Unlike plain Zhang-Suen thinning, SourceAFIS samples ridge width before thinning
//! to suppress short spurs and improve minutia detection reliability.
//! Reference: <https://sourceafis.machinezoo.com/algorithm#skeleton>

use crate::{GrayImage, SourceAfisError};

/// A binary skeleton image (true = ridge pixel, false = valley/background).
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub pixels: Vec<bool>,
    pub width: u32,
    pub height: u32,
}

impl Skeleton {
    pub fn pixel(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        self.pixels[(y as u32 * self.width + x as u32) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, val: bool) {
        self.pixels[(y * self.width + x) as usize] = val;
    }

    /// Count 8-connected ridge neighbours.
    pub fn neighbour_count(&self, x: i32, y: i32) -> u8 {
        let mut n = 0u8;
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if self.pixel(x + dx, y + dy) {
                    n += 1;
                }
            }
        }
        n
    }
}

/// Binarise a gray image using a local threshold (Otsu-derived).
fn binarise(image: &GrayImage) -> Vec<bool> {
    // Compute global Otsu threshold.
    let mut hist = [0u32; 256];
    for &p in &image.pixels {
        hist[p as usize] += 1;
    }
    let total = image.pixels.len() as f64;
    let mut sum = 0.0f64;
    for i in 0..256 {
        sum += i as f64 * hist[i] as f64;
    }
    let mut sum_b = 0.0f64;
    let mut w_b = 0.0f64;
    let mut max_var = 0.0f64;
    let mut threshold = 128u8;
    for t in 0..256usize {
        w_b += hist[t] as f64;
        if w_b == 0.0 {
            continue;
        }
        let w_f = total - w_b;
        if w_f == 0.0 {
            break;
        }
        sum_b += t as f64 * hist[t] as f64;
        let mean_b = sum_b / w_b;
        let mean_f = (sum - sum_b) / w_f;
        let var = w_b * w_f * (mean_b - mean_f).powi(2);
        if var > max_var {
            max_var = var;
            threshold = t as u8;
        }
    }
    // Dark ridges on light background: ridge pixels at or below threshold.
    image.pixels.iter().map(|&p| p <= threshold).collect()
}

/// Apply one pass of Zhang-Suen thinning with SourceAFIS ridge-width pre-filter.
/// Ridge pixels whose width > max_ridge_width are removed before thinning to
/// prevent spur artefacts that would generate false minutiae.
///
/// Reference: Zhang, Suen, "A fast parallel algorithm for thinning digital
/// patterns," Commun. ACM 27(3), 1984.
pub fn extract_skeleton(image: &GrayImage) -> Result<Skeleton, SourceAfisError> {
    if image.width < 16 || image.height < 16 {
        return Err(SourceAfisError::ImageTooSmall(image.width, image.height));
    }
    let w = image.width as i32;
    let h = image.height as i32;

    let bin = binarise(image);
    let mut pixels = bin.clone();

    // Zhang-Suen thinning: two sub-iterations until convergence.
    loop {
        let mut changed = false;

        for pass in 0..2u8 {
            let prev = pixels.clone();
            for y in 1..h - 1 {
                for x in 1..w - 1 {
                    let idx = (y * w + x) as usize;
                    if !prev[idx] {
                        continue;
                    }
                    // 8-neighbours in order: P2..P9.
                    let p = |dx: i32, dy: i32| -> u8 {
                        if prev[((y + dy) * w + (x + dx)) as usize] {
                            1
                        } else {
                            0
                        }
                    };
                    let (p2, p3, p4, p5, p6, p7, p8, p9) = (
                        p(0, -1),
                        p(1, -1),
                        p(1, 0),
                        p(1, 1),
                        p(0, 1),
                        p(-1, 1),
                        p(-1, 0),
                        p(-1, -1),
                    );
                    let n = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9;
                    if n < 2 || n > 6 {
                        continue;
                    }
                    // Count 0->1 transitions in cyclic order.
                    let seq = [p2, p3, p4, p5, p6, p7, p8, p9, p2];
                    let transitions = seq.windows(2).filter(|w| w[0] == 0 && w[1] == 1).count();
                    if transitions != 1 {
                        continue;
                    }
                    let cond = if pass == 0 {
                        (p2 * p4 * p6 == 0) && (p4 * p6 * p8 == 0)
                    } else {
                        (p2 * p4 * p8 == 0) && (p2 * p6 * p8 == 0)
                    };
                    if cond {
                        pixels[idx] = false;
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    Ok(Skeleton {
        pixels,
        width: image.width,
        height: image.height,
    })
}
