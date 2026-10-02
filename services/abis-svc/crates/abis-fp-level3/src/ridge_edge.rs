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

//! Ridge edge contour extraction.
//!
//! Reference: Jain, Chen, Demirkus, IEEE TPAMI 2007, Section III-B.
//!
//! For each ridge, the edge contour is represented as a list of EdgePoints
//! capturing curvature and ridge width along the ridge.

use serde::{Deserialize, Serialize};

/// One point on a ridge edge contour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePoint {
    pub x: f32,
    pub y: f32,
    /// Curvature at this point (higher = more curved). Unsigned.
    pub curvature: f32,
    /// Ridge width at this location in pixels.
    pub ridge_width: f32,
    /// Side of the ridge: 0 = left edge, 1 = right edge.
    pub side: u8,
}

/// Extract ridge edge contour points from an intensity image.
///
/// `intensity` is row-major [0, 255]. `orientation_field` is row-major [0, 179] degrees.
/// Returns edge points from a simplified scan: for each column-major strip, locate
/// the transition from valley to ridge (left edge) and ridge to valley (right edge).
pub fn extract_ridge_edges(
    intensity: &[u8],
    width: usize,
    height: usize,
    orientation_field: &[u8],
) -> Vec<EdgePoint> {
    if intensity.len() != width * height {
        return Vec::new();
    }

    let threshold = 128u8;
    let mut points = Vec::new();

    for y in 1..(height.saturating_sub(1)) {
        let mut in_ridge = false;
        let mut ridge_start = 0usize;

        for x in 0..width {
            let idx = y * width + x;
            let on_ridge = intensity[idx] < threshold;

            if on_ridge && !in_ridge {
                // Left edge transition.
                in_ridge = true;
                ridge_start = x;
                let curv = compute_curvature(intensity, width, height, x, y);
                let orient = if !orientation_field.is_empty() && idx < orientation_field.len() {
                    orientation_field[idx] as f32
                } else {
                    0.0
                };
                let _ = orient;
                points.push(EdgePoint {
                    x: x as f32,
                    y: y as f32,
                    curvature: curv,
                    ridge_width: 0.0, // filled below
                    side: 0,
                });
            } else if !on_ridge && in_ridge {
                // Right edge transition.
                let width_px = (x - ridge_start) as f32;
                in_ridge = false;
                let curv = compute_curvature(intensity, width, height, x - 1, y);
                points.push(EdgePoint {
                    x: (x - 1) as f32,
                    y: y as f32,
                    curvature: curv,
                    ridge_width: width_px,
                    side: 1,
                });
                // Back-fill ridge_width on the left edge point.
                if let Some(left) = points
                    .iter_mut()
                    .rev()
                    .find(|p| p.side == 0 && (p.y - y as f32).abs() < 0.5)
                {
                    left.ridge_width = width_px;
                }
            }
        }
    }
    points
}

/// Approximate curvature via second-order finite difference in both directions.
fn compute_curvature(intensity: &[u8], width: usize, height: usize, x: usize, y: usize) -> f32 {
    if x == 0 || x + 1 >= width || y == 0 || y + 1 >= height {
        return 0.0;
    }
    let v = |row: usize, col: usize| intensity[row * width + col] as f32;
    let dxx = v(y, x + 1) - 2.0 * v(y, x) + v(y, x - 1);
    let dyy = v(y + 1, x) - 2.0 * v(y, x) + v(y - 1, x);
    ((dxx * dxx + dyy * dyy).sqrt() / 255.0).clamp(0.0, 1.0)
}
