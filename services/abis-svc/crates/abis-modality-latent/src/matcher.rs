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

//! Latent minutiae matcher with quality-weighted subset scoring.

use serde::{Deserialize, Serialize};

use crate::domain::{LatentTemplate, Minutia};

const ROT_TOL_DEG: f32 = 30.0;
const TRANS_TOL_PX: f32 = 80.0;

/// Match score for a latent comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub raw: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
    pub matched_minutiae: usize,
}

/// Default threshold for latent acceptance. Lower than palm due to partial prints.
pub const DEFAULT_THRESHOLD: f32 = 0.35;

/// Compare a latent probe against a gallery template.
///
/// Score = (matched_minutiae / probe_count) * quality_weight, where
/// quality_weight = probe.usable_ridge_pct.
pub fn match_latent(probe: &LatentTemplate, gallery: &LatentTemplate) -> Score {
    if probe.minutiae.is_empty() {
        return Score {
            raw: 0.0,
            threshold: DEFAULT_THRESHOLD,
            threshold_pass: false,
            matched_minutiae: 0,
        };
    }

    let matched = count_matched(&probe.minutiae, &gallery.minutiae);
    let probe_count = probe.minutiae.len() as f32;
    let quality_weight = probe.usable_ridge_pct.clamp(0.0, 1.0);
    let raw = (matched as f32 / probe_count) * quality_weight;

    Score {
        raw: raw.min(1.0),
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
        matched_minutiae: matched,
    }
}

fn angle_diff(a: u8, b: u8) -> f32 {
    let diff = (a as i16 - b as i16).abs();
    let wrapped = if diff > 90 { 180 - diff } else { diff };
    wrapped as f32
}

fn count_matched(probe: &[Minutia], gallery: &[Minutia]) -> usize {
    let mut matched = 0usize;
    let mut gallery_used = vec![false; gallery.len()];

    for pm in probe {
        for (gi, gm) in gallery.iter().enumerate() {
            if gallery_used[gi] {
                continue;
            }
            let dx = (pm.x as f32 - gm.x as f32).abs();
            let dy = (pm.y as f32 - gm.y as f32).abs();
            let dist = (dx * dx + dy * dy).sqrt();
            let rot = angle_diff(pm.angle, gm.angle);
            if dist <= TRANS_TOL_PX && rot <= ROT_TOL_DEG {
                matched += 1;
                gallery_used[gi] = true;
                break;
            }
        }
    }

    matched
}
