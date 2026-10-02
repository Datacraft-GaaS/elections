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

//! Ridge counting between two minutiae via orientation-field traversal.
//!
//! Reference: Watson et al., NISTIR 7392, 2007, Bozorth3 ridge-count algorithm.
//!
//! Algorithm:
//!   1. Draw the straight-line path from minutia A to minutia B.
//!   2. At each sample point along the path, read the local ridge orientation.
//!   3. The effective ridge crossing density is inversely proportional to the
//!      inter-ridge spacing and proportional to sin(path_angle - ridge_angle):
//!      crossing at a given step = |sin(path_angle - ridge_angle)| / inter_ridge_spacing.
//!   4. Integrate across all steps to get the total ridge count.
//!
//! Geometric derivation: ridges are parallel lines at spacing d. Crossing N ridges
//! with a line of length L at angle alpha to the ridges: N = L * |sin(alpha)| / d.

use abis_matcher::modality::fingerprint::Minutia;

/// Per-pixel orientation field: each entry is ridge orientation in degrees [0, 180).
#[derive(Debug, Clone)]
pub struct OrientationField {
    pub width: usize,
    pub height: usize,
    /// Row-major, one value per pixel in degrees [0, 180).
    pub data: Vec<f32>,
}

impl OrientationField {
    pub fn new(width: usize, height: usize, data: Vec<f32>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    /// Return orientation at (x, y), clamped to image bounds.
    pub fn get(&self, x: usize, y: usize) -> f32 {
        let xi = x.min(self.width.saturating_sub(1));
        let yi = y.min(self.height.saturating_sub(1));
        self.data[yi * self.width + xi]
    }

    /// Bilinear interpolation of orientation at fractional (x, y).
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.width.saturating_sub(1));
        let y1 = (y0 + 1).min(self.height.saturating_sub(1));
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let v00 = self.get(x0, y0);
        let v10 = self.get(x1, y0);
        let v01 = self.get(x0, y1);
        let v11 = self.get(x1, y1);

        let top = v00 + (v10 - v00) * fx;
        let bot = v01 + (v11 - v01) * fx;
        top + (bot - top) * fy
    }
}

/// Assumed inter-ridge spacing in pixels at 500 ppi.
const INTER_RIDGE_SPACING_PX: f32 = 15.0;

/// Count the number of ridges crossed on the straight line between two minutiae.
///
/// Uses the geometric ridge-crossing formula:
///   N = integral over path of |sin(path_angle - ridge_angle(t))| / inter_ridge_spacing
///
/// This is the standard approach for Bozorth3-style ridge counting when working
/// from an orientation field rather than raw pixel intensities. At each sample
/// point the local ridge orientation is read from the field and the incremental
/// crossing density is computed.
///
/// Returns the ridge count as a u8 (capped at 255).
/// When `orientation_field` has no pixels (empty), returns a geometric estimate
/// assuming a uniform ridge spacing with ridges perpendicular to the path.
pub fn count_ridges(from: &Minutia, to: &Minutia, orientation_field: &OrientationField) -> u8 {
    let dx = to.x as f32 - from.x as f32;
    let dy = to.y as f32 - from.y as f32;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 1.0 {
        return 0;
    }

    if orientation_field.data.is_empty() {
        // Fallback: assume ridges run perpendicular to path (worst case / maximum crossings).
        return (dist / INTER_RIDGE_SPACING_PX).round().min(255.0) as u8;
    }

    // Path angle in [0, PI) using atan2, then mapped to ridge-orientation half-plane.
    let path_angle = dy.atan2(dx); // in [-PI, PI]

    let ux = dx / dist;
    let uy = dy / dist;

    // Integrate crossing density along the path at pixel-level steps.
    let steps = (dist.ceil() as usize).max(1);
    let step_size = dist / steps as f32;
    let mut acc = 0.0f32;

    for step in 0..=steps {
        let t = step as f32 * step_size;
        let sx = from.x as f32 + ux * t;
        let sy = from.y as f32 + uy * t;

        // Ridge orientation in [0, 180) degrees; convert to radians in [0, PI).
        let orient_deg = orientation_field.sample(sx, sy);
        let orient_rad = orient_deg * std::f32::consts::PI / 180.0;

        // The angle between path direction and ridge direction.
        // Ridge lines run at orient_rad; crossing them requires a component perpendicular to them.
        // Number of ridge crossings per unit length = |sin(path_angle - orient_rad)| / spacing.
        let angle_diff = path_angle - orient_rad;
        let sin_val = angle_diff.sin().abs();

        acc += sin_val * step_size / INTER_RIDGE_SPACING_PX;
    }

    acc.round().min(255.0) as u8
}
