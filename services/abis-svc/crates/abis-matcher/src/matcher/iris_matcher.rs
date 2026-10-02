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

//! Iris matcher: masked Hamming distance on 2048-bit IrisCodes.

use crate::matcher::{Matcher, Score};
use crate::modality::IrisTemplate;

/// Iris matcher using Daugman masked Hamming distance.
#[derive(Debug, Clone)]
pub struct IrisMatcher {
    /// Decision threshold (HD below this = match). Default per Daugman: 0.32.
    pub hd_threshold: f32,
    /// Minimum mask overlap fraction required. Reject if below.
    pub min_overlap: f32,
}

impl Default for IrisMatcher {
    fn default() -> Self {
        Self {
            hd_threshold: 0.32,
            min_overlap: 0.30,
        }
    }
}

impl IrisMatcher {
    pub fn new(hd_threshold: f32) -> Self {
        Self {
            hd_threshold,
            ..Self::default()
        }
    }
}

impl Matcher<IrisTemplate> for IrisMatcher {
    fn compare(&self, probe: &IrisTemplate, gallery: &IrisTemplate) -> Score {
        match probe.hamming_distance(gallery, self.min_overlap) {
            None => {
                // Insufficient mask overlap: fail with score 0.
                Score::new(f32::NAN, 0.0, self.hd_threshold)
            }
            Some(hd) => {
                // Convert HD to similarity: 0 HD = score 1.0, 0.5 HD = score 0.0.
                let normalised = (1.0 - hd / 0.5).clamp(0.0, 1.0);
                // threshold_pass: hd < hd_threshold means match.
                let threshold_pass = hd < self.hd_threshold;
                Score {
                    raw: hd,
                    normalised,
                    threshold_pass,
                }
            }
        }
    }
}
