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

//! Face matcher: cosine similarity on 512-d L2-normalised embeddings.

use crate::matcher::{Matcher, Score};
use crate::modality::FaceTemplate;

/// Configurable cohort statistics for z-score normalisation.
#[derive(Debug, Clone)]
pub struct CohortStats {
    pub mean: f32,
    pub std: f32,
}

impl Default for CohortStats {
    fn default() -> Self {
        // Empirical baseline for a diverse 30M-enrolment cohort.
        Self {
            mean: 0.35,
            std: 0.12,
        }
    }
}

/// Face matcher using cosine similarity with optional z-score normalisation.
#[derive(Debug, Clone)]
pub struct FaceMatcher {
    /// Decision threshold on the normalised score.
    pub threshold: f32,
    /// Cohort statistics for z-score normalisation. `None` = raw cosine similarity.
    pub cohort: Option<CohortStats>,
}

impl Default for FaceMatcher {
    fn default() -> Self {
        Self {
            threshold: 0.6,
            cohort: Some(CohortStats::default()),
        }
    }
}

impl FaceMatcher {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            ..Self::default()
        }
    }

    pub fn with_cohort(threshold: f32, cohort: CohortStats) -> Self {
        Self {
            threshold,
            cohort: Some(cohort),
        }
    }

    /// Cosine similarity in [0.0, 1.0] (1.0 = identical).
    pub fn cosine_similarity(&self, probe: &FaceTemplate, gallery: &FaceTemplate) -> f32 {
        probe.dot_product(gallery).clamp(0.0, 1.0)
    }

    /// Z-score normalised similarity.
    fn normalise_score(&self, raw: f32) -> f32 {
        match &self.cohort {
            None => raw,
            Some(stats) => {
                if stats.std < 1e-9 {
                    return raw;
                }
                // Shift and scale so threshold sits at 0.5 in normalised space.
                let z = (raw - stats.mean) / stats.std;
                // Map z to [0, 1] via sigmoid-like clamp.
                (z / 4.0 + 0.5).clamp(0.0, 1.0)
            }
        }
    }
}

impl Matcher<FaceTemplate> for FaceMatcher {
    fn compare(&self, probe: &FaceTemplate, gallery: &FaceTemplate) -> Score {
        let raw = self.cosine_similarity(probe, gallery);
        let normalised = self.normalise_score(raw);
        Score::new(raw, normalised, self.threshold)
    }
}
