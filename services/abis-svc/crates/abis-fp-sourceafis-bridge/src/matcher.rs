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

//! SourceAFIS matcher: seed-pair selection, radial extension, score aggregation.
//!
//! The matcher follows the three-phase pipeline documented at
//! <https://sourceafis.machinezoo.com/algorithm#matching>:
//! 1. Find a seed pair (probe minutia i, gallery minutia j) with high structure
//!    similarity.
//! 2. Radially extend: from the seed, try to pair more minutiae by transforming
//!    probe coordinates by the alignment implied by the seed.
//! 3. Aggregate: score = f(paired count, structure agreement, ridge count agreement).

use crate::minutia::{extract_minutiae, SourceAfisMinutia};
use crate::params::SourceAfisParams;
use crate::ridge_structure::{build_edge_table, edge_similarity, StructureEdge};
use crate::skeleton::extract_skeleton;
use crate::{GrayImage, SourceAfisError};
use std::f32::consts::PI;

/// Raw similarity score returned by the matcher (higher is more similar).
/// Normalised to [0.0, 1.0] by `compare_normalised`.
#[derive(Debug, Clone, Copy)]
pub struct MatchScore {
    /// Number of paired minutiae.
    pub paired: usize,
    /// Weighted structure-similarity contribution.
    pub structure_score: f32,
    /// Weighted ridge-count-agreement contribution.
    pub ridge_score: f32,
}

impl MatchScore {
    /// Combine into a single score in [0.0, 1.0].
    pub fn combined(&self) -> f32 {
        let raw = self.paired as f32 + self.structure_score + self.ridge_score;
        // Sigmoid-like normalisation: score saturates around 40 paired minutiae.
        let normalised = raw / (raw + 40.0);
        normalised.clamp(0.0, 1.0)
    }
}

/// Wrap angle to (-PI, PI].
fn wrap(a: f32) -> f32 {
    let mut a = a % (2.0 * PI);
    if a > PI {
        a -= 2.0 * PI;
    } else if a <= -PI {
        a += 2.0 * PI;
    }
    a
}

/// Try to extend pairings radially from a seed alignment.
/// Returns (paired_count, structure_score, ridge_score).
fn extend_from_seed(
    probe_min: &[SourceAfisMinutia],
    gallery_min: &[SourceAfisMinutia],
    probe_edges: &[Vec<StructureEdge>],
    gallery_edges: &[Vec<StructureEdge>],
    seed_p: usize,
    seed_g: usize,
    rotation: f32,
    params: &SourceAfisParams,
) -> (usize, f32, f32) {
    let mut paired_p = vec![false; probe_min.len()];
    let mut paired_g = vec![false; gallery_min.len()];
    let mut queue_p = vec![seed_p];
    let mut queue_g = vec![seed_g];
    paired_p[seed_p] = true;
    paired_g[seed_g] = true;

    let mut structure_score = 0.0f32;
    let mut ridge_score = 0.0f32;
    let mut paired = 1usize;

    let mut qi = 0;
    while qi < queue_p.len() {
        let pi = queue_p[qi];
        let gi = queue_g[qi];
        qi += 1;

        for pe in &probe_edges[pi] {
            if paired_p[pe.neighbour_idx] {
                continue;
            }
            // Rotate the probe edge direction by the seed alignment rotation.
            let rotated_angle = wrap(pe.relative_angle + rotation);

            // Find a matching gallery edge from gi.
            for ge in &gallery_edges[gi] {
                if paired_g[ge.neighbour_idx] {
                    continue;
                }
                // Compare directions.
                let angle_diff = wrap(rotated_angle - ge.relative_angle).abs();
                if angle_diff > params.max_angle_diff {
                    continue;
                }
                let dist_diff = (pe.distance - ge.distance).abs() / params.neighbour_radius_px;
                if dist_diff > params.max_distance_diff_frac {
                    continue;
                }
                // Direction difference between the two pointed minutiae.
                let dir_p = wrap(probe_min[pe.neighbour_idx].direction + rotation);
                let dir_g = gallery_min[ge.neighbour_idx].direction;
                let dir_diff = wrap(dir_p - dir_g).abs();
                if dir_diff > params.max_direction_diff {
                    continue;
                }

                // Match found: pair them.
                let pj = pe.neighbour_idx;
                let gj = ge.neighbour_idx;
                paired_p[pj] = true;
                paired_g[gj] = true;
                queue_p.push(pj);
                queue_g.push(gj);
                paired += 1;

                // Structure similarity contribution.
                if let Some(s) = edge_similarity(pe, ge, params, params.neighbour_radius_px) {
                    structure_score += s * params.structure_score_weight;
                }

                // Ridge count agreement.
                let rc_diff = pe.ridge_count.abs_diff(ge.ridge_count);
                if rc_diff <= params.max_ridge_count_diff {
                    ridge_score += (1.0
                        - rc_diff as f32 / (params.max_ridge_count_diff as f32 + 1.0))
                        * params.ridge_count_score_weight;
                }
                break;
            }
        }
    }
    (paired, structure_score, ridge_score)
}

/// Score two lists of minutiae (without image re-extraction).
fn score_minutiae(
    probe_min: &[SourceAfisMinutia],
    gallery_min: &[SourceAfisMinutia],
    params: &SourceAfisParams,
) -> MatchScore {
    if probe_min.is_empty() || gallery_min.is_empty() {
        return MatchScore {
            paired: 0,
            structure_score: 0.0,
            ridge_score: 0.0,
        };
    }

    let probe_edges = build_edge_table(probe_min, params);
    let gallery_edges = build_edge_table(gallery_min, params);

    // Build seed candidates: pairs with at least one matching edge in their structures.
    let mut best = MatchScore {
        paired: 0,
        structure_score: 0.0,
        ridge_score: 0.0,
    };

    'outer: for pi in 0..probe_min.len() {
        for gi in 0..gallery_min.len() {
            let dir_diff = wrap(probe_min[pi].direction - gallery_min[gi].direction).abs();
            if dir_diff > params.max_direction_diff * 2.0 {
                continue;
            }
            let rotation = gallery_min[gi].direction - probe_min[pi].direction;
            let (paired, structure_score, ridge_score) = extend_from_seed(
                probe_min,
                gallery_min,
                &probe_edges,
                &gallery_edges,
                pi,
                gi,
                rotation,
                params,
            );
            if paired > best.paired
                || (paired == best.paired && structure_score > best.structure_score)
            {
                best = MatchScore {
                    paired,
                    structure_score,
                    ridge_score,
                };
            }
            // Early termination: if we have a convincing match, stop.
            if best.combined() > 0.5 {
                break 'outer;
            }
        }
    }
    best
}

/// SourceAFIS matcher.  Accepts pre-extracted minutia lists or raw images.
pub struct SourceAfisMatcher {
    params: SourceAfisParams,
}

impl SourceAfisMatcher {
    pub fn new(params: SourceAfisParams) -> Self {
        Self { params }
    }

    /// Compare two fingerprint images.  Returns a score in [0, 1].
    pub fn compare(&self, probe: &GrayImage, gallery: &GrayImage) -> Result<f32, SourceAfisError> {
        let probe_skel = extract_skeleton(probe)?;
        let gallery_skel = extract_skeleton(gallery)?;
        let probe_min = extract_minutiae(&probe_skel);
        let gallery_min = extract_minutiae(&gallery_skel);
        let score = score_minutiae(&probe_min, &gallery_min, &self.params);
        Ok(score.combined())
    }

    /// Compare two pre-extracted minutia lists (used in benchmarks / tests).
    pub fn compare_minutiae(
        &self,
        probe: &[SourceAfisMinutia],
        gallery: &[SourceAfisMinutia],
    ) -> MatchScore {
        score_minutiae(probe, gallery, &self.params)
    }
}

impl Default for SourceAfisMatcher {
    fn default() -> Self {
        Self::new(SourceAfisParams::default())
    }
}
