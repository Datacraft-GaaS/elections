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

use crate::template::MinexTemplate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorTemplate {
    pub vendor_id: String,
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteropScore {
    pub probe_vendor: String,
    pub gallery_vendor: String,
    /// Normalized overlap score in [0.0, 1.0].
    pub score: f32,
    pub minutiae_overlap: usize,
}

/// Compute a cross-vendor interoperability score between two MINEX templates.
///
/// Matching criterion per MINEX III evaluation protocol:
///   - x/y within 15 pixels (Euclidean distance <= 15)
///   - angle within 22.5 degrees (16 units in 256-unit circle)
///
/// Score = matched_pairs / max(probe.len, gallery.len).
pub fn compute_interop_score(probe: &MinexTemplate, gallery: &MinexTemplate) -> InteropScore {
    let angle_tolerance: u16 = 16; // 22.5 deg in 256-unit circle
    let pixel_tolerance_sq: u32 = 15 * 15;

    let mut overlap = 0usize;
    let mut gallery_used = vec![false; gallery.minutiae.len()];

    for pm in &probe.minutiae {
        for (gi, gm) in gallery.minutiae.iter().enumerate() {
            if gallery_used[gi] {
                continue;
            }
            let dx = (pm.x as i32 - gm.x as i32).pow(2) as u32;
            let dy = (pm.y as i32 - gm.y as i32).pow(2) as u32;
            if dx + dy > pixel_tolerance_sq {
                continue;
            }
            let angle_diff = (pm.angle as i16 - gm.angle as i16).unsigned_abs();
            let angle_diff = angle_diff.min(256 - angle_diff);
            if angle_diff <= angle_tolerance {
                overlap += 1;
                gallery_used[gi] = true;
                break;
            }
        }
    }

    let denom = probe.minutiae.len().max(gallery.minutiae.len()).max(1);
    let score = overlap as f32 / denom as f32;

    InteropScore {
        probe_vendor: String::new(),
        gallery_vendor: String::new(),
        score,
        minutiae_overlap: overlap,
    }
}
