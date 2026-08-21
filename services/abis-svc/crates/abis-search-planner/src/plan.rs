//! Query planner: picks the cheapest step ordering.

use tracing::debug;

use crate::domain::{ExecutionPlan, PlanStep, ShardSpec, StepKind};
use crate::error::PlannerError;
use crate::estimator::CostModel;

const DEFAULT_ORDER: &[StepKind] = &[
    StepKind::DemographicPrune,
    StepKind::FaceAnn,
    StepKind::FingerprintExact,
    StepKind::IrisExact,
];

pub struct QueryPlanner {
    pub cost_model: CostModel,
}

impl Default for QueryPlanner {
    fn default() -> Self {
        Self {
            cost_model: CostModel::default(),
        }
    }
}

impl QueryPlanner {
    pub fn plan(
        &self,
        shards: Vec<ShardSpec>,
        early_termination_threshold: f64,
    ) -> Result<ExecutionPlan, PlannerError> {
        if shards.is_empty() {
            return Err(PlannerError::NoShards);
        }
        if !(0.0..1.0).contains(&early_termination_threshold) {
            return Err(PlannerError::InvalidThreshold(early_termination_threshold));
        }

        let mut available: Vec<StepKind> = DEFAULT_ORDER
            .iter()
            .filter(|s| shards.iter().any(|sh| sh.available_steps.contains(s)))
            .cloned()
            .collect();

        if available.is_empty() {
            return Err(PlannerError::NoSteps);
        }

        let max_gallery = shards.iter().map(|s| s.gallery_size).max().unwrap_or(1);
        available.sort_by(|a, b| {
            let ca = self.cost_model.estimate_step_cost(a, max_gallery);
            let cb = self.cost_model.estimate_step_cost(b, max_gallery);
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let steps: Vec<PlanStep> = available
            .into_iter()
            .map(|k| self.cost_model.build_step(k, max_gallery))
            .collect();
        let total_cost: f64 = steps.iter().map(|s| s.estimated_cost_ms).sum();
        debug!(total_cost, n_steps = steps.len(), "query plan built");

        Ok(ExecutionPlan {
            steps,
            shards,
            total_estimated_cost_ms: total_cost,
            early_termination_threshold,
        })
    }
}
