//! Error types for search planner.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlannerError {
    #[error("no shards available")]
    NoShards,

    #[error("no executable steps for query")]
    NoSteps,

    #[error("early termination threshold {0} not in [0, 1)")]
    InvalidThreshold(f64),

    #[error("execution error on shard {shard}: {reason}")]
    ShardExecution { shard: uuid::Uuid, reason: String },
}
