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

//! Incipient ridge detection.
//!
//! Reference: Jain, Chen, Demirkus, IEEE TPAMI 2007, Section III-C.
//!
//! Incipient ridges are nascent ridges that appear between mature ridges.
//! They are shorter, narrower, and have lower contrast than mature ridges.

use serde::{Deserialize, Serialize};

/// A detected incipient (nascent) ridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncipientRidge {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    /// Confidence score [0.0, 1.0].
    pub confidence: f32,
    /// Approximate ridge length in pixels.
    pub length_px: f32,
}

/// Maximum length of an incipient ridge (beyond this it is a mature ridge).
const MAX_INCIPIENT_LENGTH_PX: f32 = 60.0;
/// Maximum ridge width for incipient ridges in pixels.
const MAX_INCIPIENT_WIDTH_PX: f32 = 5.0;
/// Intensity threshold: incipient ridges are lighter than mature ridges.
const INCIPIENT_MIN_CONTRAST: f32 = 0.08;

/// Detect incipient ridges between mature ridges in an intensity image.
///
/// `intensity` is row-major [0, 255]. `width` and `height` define image dimensions.
/// Returns incipient ridges sorted by confidence descending.
///
/// Detection strategy: scan horizontal strips for short dark segments that are
/// narrower and lighter than mature ridges, bounded by mature ridge regions.
pub fn detect_incipient_ridges(
    intensity: &[u8],
    width: usize,
    height: usize,
) -> Vec<IncipientRidge> {
    if intensity.len() != width * height {
        return Vec::new();
    }

    let mature_threshold = 100u8; // mature ridge intensity
    let incipient_threshold = 160u8; // lighter than mature but darker than valley

    let mut ridges = Vec::new();

    for y in 1..(height.saturating_sub(1)) {
        let mut in_incipient = false;
        let mut seg_start = 0usize;
        let mut seg_min_intensity = 255u8;

        for x in 0..width {
            let val = intensity[y * width + x];
            let is_incipient_candidate = val >= mature_threshold && val < incipient_threshold;

            if is_incipient_candidate && !in_incipient {
                in_incipient = true;
                seg_start = x;
                seg_min_intensity = val;
            } else if is_incipient_candidate && in_incipient {
                if val < seg_min_intensity {
                    seg_min_intensity = val;
                }
            } else if !is_incipient_candidate && in_incipient {
                in_incipient = false;
                let seg_len = (x - seg_start) as f32;
                if seg_len > 1.0 && seg_len <= MAX_INCIPIENT_LENGTH_PX {
                    let width_estimate = seg_len;
                    if width_estimate <= MAX_INCIPIENT_WIDTH_PX {
                        let contrast =
                            (incipient_threshold as f32 - seg_min_intensity as f32) / 255.0;
                        if contrast >= INCIPIENT_MIN_CONTRAST {
                            ridges.push(IncipientRidge {
                                start_x: seg_start as f32,
                                start_y: y as f32,
                                end_x: (x - 1) as f32,
                                end_y: y as f32,
                                confidence: contrast.clamp(0.0, 1.0),
                                length_px: seg_len,
                            });
                        }
                    }
                }
            }
        }
    }

    ridges.sort_unstable_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ridges
}
