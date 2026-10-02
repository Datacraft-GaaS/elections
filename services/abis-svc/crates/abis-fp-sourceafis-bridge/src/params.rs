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

//! SourceAFIS algorithm constants.
//!
//! Values from Robert Vaszary's documented tuning at
//! <https://sourceafis.machinezoo.com/algorithm> and the reference Java
//! implementation (SourceAFIS 3.x, MPL-2.0).

use std::f32::consts::PI;

/// Tuning constants for the SourceAFIS matching pipeline.
#[derive(Debug, Clone)]
pub struct SourceAfisParams {
    /// Maximum pixel distance between two minutiae to be considered neighbours.
    /// SourceAFIS default: 490 / 500 ppi ~ 62 px (scaled linearly with DPI).
    pub neighbour_radius_px: f32,

    /// Minimum number of paired minutiae in seed search to accept a seed.
    pub min_seed_support: usize,

    /// Maximum directional difference (radians) for a minutia pair to be paired.
    pub max_direction_diff: f32,

    /// Maximum distance difference (fraction of neighbour_radius_px) for a structure
    /// edge to be considered matching between probe and gallery.
    pub max_distance_diff_frac: f32,

    /// Maximum angle difference between two structure edges.
    pub max_angle_diff: f32,

    /// Maximum ridge-count difference for two inter-minutia edges to match.
    pub max_ridge_count_diff: u8,

    /// Number of top-scoring seed candidates to try before accepting best.
    pub seed_candidates: usize,

    /// Score multiplier applied per paired minutia with matching local structure.
    pub structure_score_weight: f32,

    /// Score multiplier applied per paired minutia with matching ridge count.
    pub ridge_count_score_weight: f32,
}

impl Default for SourceAfisParams {
    fn default() -> Self {
        Self {
            neighbour_radius_px: 490.0,
            min_seed_support: 1,
            max_direction_diff: PI / 4.0,
            max_distance_diff_frac: 0.3,
            max_angle_diff: PI / 6.0,
            max_ridge_count_diff: 1,
            seed_candidates: 10,
            structure_score_weight: 0.032,
            ridge_count_score_weight: 0.016,
        }
    }
}
