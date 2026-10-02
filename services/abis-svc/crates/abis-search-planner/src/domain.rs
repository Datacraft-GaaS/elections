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

//! Domain types for search planner.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepKind {
    DemographicPrune,
    FaceAnn,
    FingerprintExact,
    IrisExact,
}

impl std::fmt::Display for StepKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepKind::DemographicPrune => write!(f, "demographic_prune"),
            StepKind::FaceAnn => write!(f, "face_ann"),
            StepKind::FingerprintExact => write!(f, "fingerprint_exact"),
            StepKind::IrisExact => write!(f, "iris_exact"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub kind: StepKind,
    pub estimated_cost_ms: f64,
    pub selectivity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardSpec {
    pub shard_id: Uuid,
    pub gallery_size: usize,
    pub available_steps: Vec<StepKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub steps: Vec<PlanStep>,
    pub shards: Vec<ShardSpec>,
    pub total_estimated_cost_ms: f64,
    pub early_termination_threshold: f64,
}
