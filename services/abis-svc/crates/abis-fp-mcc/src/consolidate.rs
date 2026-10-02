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

//! Global similarity via LSSR (Local Similarity Sort Relaxation).
//!
//! Reference: Cappelli, Ferrara, Maltoni, IEEE TPAMI 2010, Section III-C.
//!
//! Algorithm:
//!  1. Compute all pairwise local similarities between probe and gallery cylinders.
//!  2. Sort descending by similarity.
//!  3. Greedily select pairs with no row/column conflict (one-to-one mapping).
//!  4. Return the mean similarity of selected pairs, clamped to [0.0, 1.0].

use crate::cylinder::MinutiaCylinder;
use crate::similarity::local_similarity;

/// Global matching score via LSSR.
///
/// `probe_cyls` and `gallery_cyls` are the cylinders built from the two templates.
/// Only valid cylinders (flagged `valid = true`) are used.
/// Returns a score in [0.0, 1.0].
pub fn global_similarity(
    probe_cyls: &[MinutiaCylinder],
    gallery_cyls: &[MinutiaCylinder],
    min_matched: u8,
) -> f32 {
    let probe_valid: Vec<&MinutiaCylinder> = probe_cyls.iter().filter(|c| c.valid).collect();
    let gallery_valid: Vec<&MinutiaCylinder> = gallery_cyls.iter().filter(|c| c.valid).collect();

    if probe_valid.is_empty() || gallery_valid.is_empty() {
        return 0.0;
    }

    // Build all pairs (pi, gi, sim).
    let mut pairs: Vec<(usize, usize, f32)> = probe_valid
        .iter()
        .enumerate()
        .flat_map(|(pi, pc)| {
            gallery_valid
                .iter()
                .enumerate()
                .map(move |(gi, gc)| (pi, gi, local_similarity(pc, gc)))
        })
        .collect();

    // Sort descending by similarity.
    pairs.sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Greedy one-to-one selection.
    let mut probe_used = vec![false; probe_valid.len()];
    let mut gallery_used = vec![false; gallery_valid.len()];
    let mut selected = Vec::new();

    for (pi, gi, sim) in &pairs {
        if !probe_used[*pi] && !gallery_used[*gi] {
            probe_used[*pi] = true;
            gallery_used[*gi] = true;
            selected.push(*sim);
        }
    }

    if selected.len() < min_matched as usize {
        return 0.0;
    }

    let sum: f32 = selected.iter().sum();
    (sum / selected.len() as f32).clamp(0.0, 1.0)
}
