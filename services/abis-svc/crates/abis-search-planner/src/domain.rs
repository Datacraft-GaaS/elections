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
