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

use crate::domain::PeriocularTemplate;

pub const DEFAULT_THRESHOLD: f32 = 0.45;

pub struct Score {
    pub raw: f32,
    pub chi_sq_distance: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
}

pub fn match_periocular(probe: &PeriocularTemplate, gallery: &PeriocularTemplate) -> Score {
    let chi_sq = chi_square(&probe.lbp_histogram.bins, &gallery.lbp_histogram.bins);
    // Convert distance to similarity: higher distance means less similar.
    let raw = (-chi_sq / 2.0).exp().clamp(0.0, 1.0);
    Score {
        raw,
        chi_sq_distance: chi_sq,
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
    }
}

pub fn chi_square(h1: &[f32], h2: &[f32]) -> f32 {
    const EPS: f32 = 1e-10;
    h1.iter()
        .zip(h2.iter())
        .map(|(&a, &b)| {
            let diff = a - b;
            diff * diff / (a + b + EPS)
        })
        .sum()
}
