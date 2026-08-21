//! Plan executor: runs steps across shards with early-termination.

use tracing::info;
use uuid::Uuid;

use crate::domain::ExecutionPlan;
use crate::error::PlannerError;

#[derive(Debug, Clone)]
pub struct ShardResult {
    pub shard_id: Uuid,
    pub top_score: f64,
    pub candidate_count: usize,
}

/// Execute a plan across shards (shard_scores is (shard_id, top_score) input).
/// Stops early if any shard's score >= early_termination_threshold.
pub fn execute_plan(
    plan: &ExecutionPlan,
    shard_scores: &[(Uuid, f64)],
) -> Result<Vec<ShardResult>, PlannerError> {
    let mut results = Vec::new();
    let threshold = plan.early_termination_threshold;

    for (shard_id, top_score) in shard_scores {
        if plan.shards.iter().all(|s| s.shard_id != *shard_id) {
            continue;
        }
        results.push(ShardResult {
            shard_id: *shard_id,
            top_score: *top_score,
            candidate_count: if *top_score > 0.5 { 1 } else { 0 },
        });
        if *top_score >= threshold {
            info!(shard_id = %shard_id, top_score, "early termination triggered");
            break;
        }
    }

    Ok(results)
}
