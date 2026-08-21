//! Error types for drift monitoring.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DriftError {
    #[error("bin count mismatch: expected {expected}, got {got}")]
    BinCountMismatch { expected: usize, got: usize },

    #[error("empty distribution")]
    EmptyDistribution,

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
