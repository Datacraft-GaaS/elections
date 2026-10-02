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

//! Singular point detection via the Poincare index.
//!
//! Reference: M. Kawagoe, A. Tojo, "Fingerprint pattern classification,"
//! Pattern Recognition, vol. 17, no. 3, pp. 295-303, 1984.
//!
//! The Poincare index P(C) of a closed curve C in the orientation field equals
//!   P = (1/pi) * sum of successive orientation differences along C,
//! where each difference is wrapped to (-pi/2, pi/2].
//!   P =  1 -> core
//!   P = -1 -> delta
//!   P =  2 -> double-core (whorl)

use std::f32::consts::PI;

use crate::field::OrientationField;

/// Type of a singular point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SingularKind {
    Core,
    Delta,
    DoubleCore,
}

/// A detected singular point in the orientation field.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SingularPoint {
    pub kind: SingularKind,
    /// Column in block coordinates.
    pub x: u32,
    /// Row in block coordinates.
    pub y: u32,
    /// Dominant orientation at this point.
    pub orientation: f32,
}

/// Detect singular points in the orientation field.
///
/// Iterates over all interior blocks and computes the Poincare index on the
/// 8-block closed curve around each.  Singular points are returned in
/// left-to-right, top-to-bottom order.
pub fn detect_singular_points(field: &OrientationField) -> Vec<SingularPoint> {
    let mut result = Vec::new();
    let w = field.width as i32;
    let h = field.height as i32;

    for by in 1..h - 1 {
        for bx in 1..w - 1 {
            // 8-block clockwise loop around (bx, by).
            let loop_blocks = [
                (bx - 1, by - 1),
                (bx, by - 1),
                (bx + 1, by - 1),
                (bx + 1, by),
                (bx + 1, by + 1),
                (bx, by + 1),
                (bx - 1, by + 1),
                (bx - 1, by),
            ];
            let angles: [f32; 8] =
                std::array::from_fn(|i| field.angle_at(loop_blocks[i].0, loop_blocks[i].1));
            let pi_index = poincare_index(&angles);

            let kind = if (pi_index - 1.0).abs() < 0.5 {
                Some(SingularKind::Core)
            } else if (pi_index + 1.0).abs() < 0.5 {
                Some(SingularKind::Delta)
            } else if (pi_index - 2.0).abs() < 0.5 {
                Some(SingularKind::DoubleCore)
            } else {
                None
            };

            if let Some(k) = kind {
                result.push(SingularPoint {
                    kind: k,
                    x: bx as u32,
                    y: by as u32,
                    orientation: field.angle_at(bx, by),
                });
            }
        }
    }

    result
}

/// Compute the Poincare index (in units of 1 turn, i.e. integer values for singular points).
fn poincare_index(angles: &[f32; 8]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..8 {
        let diff = wrap_orientation_diff(angles[(i + 1) % 8] - angles[i]);
        sum += diff;
    }
    sum / PI
}

/// Wrap an orientation difference to (-PI/2, PI/2].
fn wrap_orientation_diff(mut d: f32) -> f32 {
    while d > PI / 2.0 {
        d -= PI;
    }
    while d <= -PI / 2.0 {
        d += PI;
    }
    d
}
