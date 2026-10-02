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

//! 2D Gabor filter bank for IrisCode generation.
//!
//! Reference: Daugman, J. (1993). High confidence visual recognition of persons by a test of
//! statistical independence. IEEE TPAMI, 15(11), 1148-1161.
//!
//! A 2D Gabor wavelet is:
//!   G(x, y, f, theta, sigma_x, sigma_y) = exp(-x'^2/(2*sigma_x^2) - y'^2/(2*sigma_y^2))
//!                                         * cos(2*pi*f*x')
//! where x' = x*cos(theta) + y*sin(theta), y' = -x*sin(theta) + y*cos(theta).

use crate::normalise::NormalisedIris;

/// A single 2D Gabor filter kernel.
#[derive(Debug, Clone)]
pub struct GaborFilter {
    /// Spatial frequency (cycles per pixel).
    pub frequency: f32,
    /// Orientation in radians.
    pub orientation: f32,
    /// Standard deviation along x' axis.
    pub sigma_x: f32,
    /// Standard deviation along y' axis.
    pub sigma_y: f32,
    /// Kernel half-width (kernel is 2*half+1 square).
    pub half_size: u32,
}

impl GaborFilter {
    /// Evaluate the Gabor response at image column `col`, row `row` in the normalised iris.
    ///
    /// Returns the real part of the complex Gabor response (cosine phase).
    pub fn respond(&self, norm: &NormalisedIris, col: u32, row: u32) -> f32 {
        let h = self.half_size as i32;
        let mut real = 0.0f32;

        for dy in -h..=h {
            for dx in -h..=h {
                let sx = col as i32 + dx;
                let sy = row as i32 + dy;

                // Clamp to valid range.
                let sx = sx.clamp(0, NormalisedIris::WIDTH as i32 - 1) as u32;
                let sy = sy.clamp(0, NormalisedIris::HEIGHT as i32 - 1) as u32;
                let pixel = norm.get(sx, sy) as f32 / 255.0;

                let xp = dx as f32 * self.orientation.cos() + dy as f32 * self.orientation.sin();
                let yp = -dx as f32 * self.orientation.sin() + dy as f32 * self.orientation.cos();

                let envelope = (-xp * xp / (2.0 * self.sigma_x * self.sigma_x)
                    - yp * yp / (2.0 * self.sigma_y * self.sigma_y))
                    .exp();
                let carrier = (2.0 * std::f32::consts::PI * self.frequency * xp).cos();
                real += pixel * envelope * carrier;
            }
        }

        real
    }
}

/// Standard Daugman Gabor filter bank: 8 frequencies x 4 orientations = 32 filters.
pub fn standard_filter_bank() -> Vec<GaborFilter> {
    let frequencies = [0.1f32, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45];
    let orientations = [
        0.0f32,
        std::f32::consts::PI / 4.0,
        std::f32::consts::PI / 2.0,
        3.0 * std::f32::consts::PI / 4.0,
    ];

    let mut filters = Vec::new();
    for &freq in &frequencies {
        for &orient in &orientations {
            filters.push(GaborFilter {
                frequency: freq,
                orientation: orient,
                sigma_x: 3.0,
                sigma_y: 1.5,
                half_size: 4,
            });
        }
    }
    filters
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform_norm(val: u8) -> NormalisedIris {
        let data = vec![val; (NormalisedIris::WIDTH * NormalisedIris::HEIGHT) as usize];
        NormalisedIris { data }
    }

    #[test]
    fn filter_bank_has_32_filters() {
        assert_eq!(standard_filter_bank().len(), 32);
    }

    #[test]
    fn gabor_response_finite_on_uniform_image() {
        let norm = uniform_norm(128);
        let filt = standard_filter_bank().into_iter().next().expect("filter");
        let resp = filt.respond(&norm, 50, 10);
        assert!(resp.is_finite());
    }

    #[test]
    fn gabor_response_zero_on_black_image() {
        // When all pixels are 0, response should be 0.
        let norm = uniform_norm(0);
        let filt = standard_filter_bank().into_iter().next().expect("filter");
        let resp = filt.respond(&norm, 50, 10);
        assert!(resp.abs() < 1e-6, "expected ~0 on black image, got {resp}");
    }
}
