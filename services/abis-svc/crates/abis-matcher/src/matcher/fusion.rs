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

//! Multi-modal score fusion.

use crate::modality::Modality;

/// Per-modality weight configuration.
#[derive(Debug, Clone)]
pub struct FusionPolicy {
    pub fingerprint_weight: f32,
    pub face_weight: f32,
    pub iris_weight: f32,
}

impl Default for FusionPolicy {
    fn default() -> Self {
        Self {
            fingerprint_weight: 0.5,
            face_weight: 0.3,
            iris_weight: 0.2,
        }
    }
}

impl FusionPolicy {
    pub fn weight_for(&self, modality: Modality) -> f32 {
        match modality {
            Modality::Fingerprint => self.fingerprint_weight,
            Modality::Face => self.face_weight,
            Modality::Iris => self.iris_weight,
        }
    }
}

/// Multi-modal fused score.
#[derive(Debug, Clone)]
pub struct FusionResult {
    /// Weighted sum of per-modality normalised scores.
    pub fused: f32,
    /// Confidence increases with the number of contributing modalities.
    pub confidence: f32,
    /// (modality, normalised_score) pairs that contributed.
    pub contributors: Vec<(Modality, f32)>,
}

/// Per-modality input to the fusion engine.
#[derive(Debug, Clone)]
pub struct MultiModalScore {
    pub modality: Modality,
    /// Normalised score from the per-modality matcher [0.0, 1.0].
    pub normalised: f32,
}

impl MultiModalScore {
    pub fn new(modality: Modality, normalised: f32) -> Self {
        Self {
            modality,
            normalised,
        }
    }
}

/// Fuse a set of per-modality scores under the given policy.
pub fn fuse(scores: &[MultiModalScore], policy: &FusionPolicy) -> FusionResult {
    if scores.is_empty() {
        return FusionResult {
            fused: 0.0,
            confidence: 0.0,
            contributors: vec![],
        };
    }

    let mut weighted_sum = 0.0f32;
    let mut total_weight = 0.0f32;
    let mut contributors = Vec::with_capacity(scores.len());

    for s in scores {
        let w = policy.weight_for(s.modality);
        weighted_sum += s.normalised * w;
        total_weight += w;
        contributors.push((s.modality, s.normalised));
    }

    let fused = if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        0.0
    };

    // Confidence: fraction of modalities contributing × weight-coverage.
    let max_modalities = 3.0f32;
    let modal_fraction = scores.len() as f32 / max_modalities;
    let weight_fraction = (total_weight
        / (policy.fingerprint_weight + policy.face_weight + policy.iris_weight))
        .min(1.0);
    let confidence = (modal_fraction * 0.5 + weight_fraction * 0.5).min(1.0);

    FusionResult {
        fused,
        confidence,
        contributors,
    }
}
