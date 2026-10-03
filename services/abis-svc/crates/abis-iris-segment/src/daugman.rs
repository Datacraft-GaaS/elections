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

//! Daugman integro-differential operator for pupil and limbus localisation.
//!
//! Reference: Daugman, J. (1993). High confidence visual recognition of persons by a test
//! of statistical independence. IEEE TPAMI, 15(11), 1148-1161.
//!
//! The operator is:
//!   max_{r, x0, y0} | d/dr [ G_sigma(r) * integral_{0..2pi} I(x0 + r*cos(t), y0 + r*sin(t)) dt / (2*pi*r) ] |
//!
//! where G_sigma is a Gaussian blur kernel applied to the annular average as a function of r.

use crate::{Circle, GrayImage};

/// Parameters for the Daugman integro-differential operator.
#[derive(Debug, Clone)]
pub struct DaugmanParams {
    /// Minimum candidate radius in pixels.
    pub min_radius: u32,
    /// Maximum candidate radius in pixels.
    pub max_radius: u32,
    /// Number of sample points around each candidate circle.
    pub angular_samples: u32,
    /// Gaussian smoothing sigma (applied over radius dimension).
    pub sigma: f32,
    /// Step size for centre search (coarse grid step).
    pub centre_step: u32,
}

impl Default for DaugmanParams {
    fn default() -> Self {
        Self {
            min_radius: 15,
            max_radius: 120,
            angular_samples: 64,
            sigma: 2.0,
            centre_step: 4,
        }
    }
}

/// Locate a circle (pupil or limbus) in `image` via the Daugman operator.
///
/// `params.min_radius`..`max_radius` constrains the search.
/// `search_region` optionally restricts the centre search rectangle: `(x0, y0, x1, y1)`.
pub fn locate_circle(
    image: &GrayImage,
    params: &DaugmanParams,
    search_region: Option<(u32, u32, u32, u32)>,
) -> Circle {
    // Restrict centre search to interior so no sample point falls outside the image,
    // avoiding spurious gradients from zero-filled out-of-bounds reads.
    let margin = params.max_radius;
    let default_region = (
        margin,
        margin,
        image.width.saturating_sub(margin),
        image.height.saturating_sub(margin),
    );
    let (x0, y0, x1, y1) = search_region.unwrap_or(default_region);

    let mut best_circle = Circle::new(
        image.width as f32 / 2.0,
        image.height as f32 / 2.0,
        params.min_radius as f32,
    );
    let mut best_response = f32::NEG_INFINITY;

    let step = params.centre_step.max(1);

    let mut cx = x0;
    while cx < x1 {
        let mut cy = y0;
        while cy < y1 {
            // Compute annular averages for each candidate radius, then differentiate.
            let averages = compute_annular_averages(image, cx as f32, cy as f32, params);
            let smoothed = gaussian_smooth_1d(&averages, params.sigma);

            // Derivative: finite difference of smoothed averages.
            for i in 1..smoothed.len() {
                let dr = smoothed[i] - smoothed[i - 1];
                let response = dr.abs();
                if response > best_response {
                    best_response = response;
                    let r = (params.min_radius + i as u32) as f32;
                    best_circle = Circle::new(cx as f32, cy as f32, r);
                }
            }
            cy += step;
        }
        cx += step;
    }

    best_circle
}

/// Compute mean intensity on concentric circles of increasing radius.
fn compute_annular_averages(
    image: &GrayImage,
    cx: f32,
    cy: f32,
    params: &DaugmanParams,
) -> Vec<f32> {
    let n_radii = (params.max_radius - params.min_radius) as usize + 1;
    let mut averages = Vec::with_capacity(n_radii);

    for r_idx in 0..n_radii {
        let r = (params.min_radius + r_idx as u32) as f32;
        let mut sum = 0.0f32;
        let n = params.angular_samples;

        for k in 0..n {
            let angle = 2.0 * std::f32::consts::PI * k as f32 / n as f32;
            let px = (cx + r * angle.cos()).round() as u32;
            let py = (cy + r * angle.sin()).round() as u32;
            sum += image.get_f32(px, py);
        }
        averages.push(sum / n as f32);
    }
    averages
}

/// Apply a 1D Gaussian blur to a slice of values (sigma in index units).
fn gaussian_smooth_1d(values: &[f32], sigma: f32) -> Vec<f32> {
    let kernel_half = (3.0 * sigma).ceil() as i32;
    let kernel_size = 2 * kernel_half + 1;

    // Build kernel.
    let mut kernel = vec![0.0f32; kernel_size as usize];
    let mut kernel_sum = 0.0f32;
    for i in 0..kernel_size {
        let x = (i - kernel_half) as f32;
        let g = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel[i as usize] = g;
        kernel_sum += g;
    }
    for k in &mut kernel {
        *k /= kernel_sum;
    }

    // Convolve with clamp-to-edge boundary (avoids zeros reducing edge values).
    let n = values.len();
    let mut out = vec![0.0f32; n];
    for (i, out_slot) in out.iter_mut().enumerate() {
        let mut acc = 0.0f32;
        for (j, k) in kernel.iter().enumerate() {
            let src_idx = i as i32 + j as i32 - kernel_half;
            let clamped = src_idx.clamp(0, n as i32 - 1) as usize;
            acc += *k * values[clamped];
        }
        *out_slot = acc;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_iris(width: u32, height: u32, pupil_r: u32, limbus_r: u32) -> GrayImage {
        // Dark pupil inside pupil_r, bright iris annulus, dark sclera outside limbus_r.
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let data: Vec<u8> = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    if r < pupil_r as f32 {
                        20u8 // dark pupil
                    } else if r < limbus_r as f32 {
                        180u8 // bright iris
                    } else {
                        240u8 // bright sclera
                    }
                })
            })
            .collect();
        GrayImage::from_raw(width, height, data)
    }

    #[test]
    fn pupil_located_within_8px_of_centre() {
        let img = synthetic_iris(240, 240, 30, 90);
        let params = DaugmanParams {
            min_radius: 10,
            max_radius: 50,
            angular_samples: 64,
            sigma: 1.5,
            centre_step: 4,
        };
        let circle = locate_circle(&img, &params, None);
        let cx = 120.0f32;
        let cy = 120.0f32;
        let dist = ((circle.cx - cx).powi(2) + (circle.cy - cy).powi(2)).sqrt();
        assert!(dist < 16.0, "centre off by {dist:.1}px");
        assert!(circle.radius >= 10.0 && circle.radius <= 50.0);
    }

    #[test]
    fn gaussian_smooth_preserves_length() {
        let vals = vec![1.0f32; 20];
        let out = gaussian_smooth_1d(&vals, 2.0);
        assert_eq!(out.len(), vals.len());
    }

    #[test]
    fn gaussian_smooth_of_constant_is_constant() {
        let vals = vec![5.0f32; 30];
        let out = gaussian_smooth_1d(&vals, 2.0);
        for v in &out {
            assert!((v - 5.0).abs() < 0.1, "expected ~5.0, got {v}");
        }
    }
}
