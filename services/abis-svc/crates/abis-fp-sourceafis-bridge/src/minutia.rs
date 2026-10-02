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

//! SourceAFIS minutia representation and extraction.
//!
//! Each minutia stores position, direction, type, and a pre-computed local ridge
//! structure descriptor.  The descriptor enables fast seed-pair selection during
//! matching.
//!
//! Reference: <https://sourceafis.machinezoo.com/algorithm#minutiae>

use crate::skeleton::Skeleton;
use std::f32::consts::PI;

/// Minutia type (ISO/IEC 19794-2 Table 4, adapted for SourceAFIS skeleton analysis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinutiaType {
    /// Ridge ending: a skeleton pixel with exactly one neighbour.
    Ending,
    /// Bifurcation: a skeleton pixel with exactly three neighbours.
    Bifurcation,
}

/// A single SourceAFIS minutia with full local descriptor.
#[derive(Debug, Clone)]
pub struct SourceAfisMinutia {
    pub x: f32,
    pub y: f32,
    /// Ridge direction at the minutia (radians in [0, 2*PI)).
    pub direction: f32,
    pub minutia_type: MinutiaType,
}

/// Estimate the ridge direction at a skeleton pixel by following the ridge.
fn ridge_direction(skeleton: &Skeleton, x: i32, y: i32) -> f32 {
    // Follow the ridge for up to 8 steps to get a stable direction.
    let mut cx = x;
    let mut cy = y;
    let mut prev_x = x;
    let mut prev_y = y;
    for _ in 0..8 {
        let mut found = false;
        'outer: for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = cx + dx;
                let ny = cy + dy;
                if nx == prev_x && ny == prev_y {
                    continue;
                }
                if skeleton.pixel(nx, ny) {
                    prev_x = cx;
                    prev_y = cy;
                    cx = nx;
                    cy = ny;
                    found = true;
                    break 'outer;
                }
            }
        }
        if !found {
            break;
        }
    }
    let dx = (cx - x) as f32;
    let dy = (cy - y) as f32;
    let angle = dy.atan2(dx);
    // Normalise to [0, 2*PI).
    if angle < 0.0 {
        angle + 2.0 * PI
    } else {
        angle
    }
}

/// Extract minutiae from a thinned skeleton.
pub fn extract_minutiae(skeleton: &Skeleton) -> Vec<SourceAfisMinutia> {
    let mut minutiae = Vec::new();
    let w = skeleton.width as i32;
    let h = skeleton.height as i32;

    // Skip 2-pixel border to avoid edge artefacts.
    for y in 2..h - 2 {
        for x in 2..w - 2 {
            if !skeleton.pixel(x, y) {
                continue;
            }
            let n = skeleton.neighbour_count(x, y);
            let minutia_type = match n {
                1 => MinutiaType::Ending,
                3 => MinutiaType::Bifurcation,
                _ => continue,
            };
            let direction = ridge_direction(skeleton, x, y);
            minutiae.push(SourceAfisMinutia {
                x: x as f32,
                y: y as f32,
                direction,
                minutia_type,
            });
        }
    }
    minutiae
}
