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

//! Cost model with cardinality estimates and selectivity factors.

use crate::domain::{PlanStep, StepKind};

pub struct CostModel {
    pub demographic_prune_ms_per_m: f64,
    pub face_ann_ms_per_m: f64,
    pub fingerprint_exact_ms_per_m: f64,
    pub iris_exact_ms_per_m: f64,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            demographic_prune_ms_per_m: 5.0,
            face_ann_ms_per_m: 50.0,
            fingerprint_exact_ms_per_m: 200.0,
            iris_exact_ms_per_m: 150.0,
        }
    }
}

impl CostModel {
    pub fn estimate_step_cost(&self, step: &StepKind, gallery_size: usize) -> f64 {
        let m = gallery_size as f64 / 1_000_000.0;
        match step {
            StepKind::DemographicPrune => self.demographic_prune_ms_per_m * m,
            StepKind::FaceAnn => self.face_ann_ms_per_m * m,
            StepKind::FingerprintExact => self.fingerprint_exact_ms_per_m * m,
            StepKind::IrisExact => self.iris_exact_ms_per_m * m,
        }
    }

    pub fn build_step(&self, kind: StepKind, gallery_size: usize) -> PlanStep {
        let cost = self.estimate_step_cost(&kind, gallery_size);
        let selectivity = match &kind {
            StepKind::DemographicPrune => 0.10,
            StepKind::FaceAnn => 0.001,
            StepKind::FingerprintExact => 0.0001,
            StepKind::IrisExact => 0.0001,
        };
        PlanStep {
            kind,
            estimated_cost_ms: cost,
            selectivity,
        }
    }
}
