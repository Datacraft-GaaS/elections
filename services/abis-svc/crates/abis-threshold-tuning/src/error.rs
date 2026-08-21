//! Error types for threshold tuning.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThresholdError {
    #[error("insufficient labelled pairs: need >= {min}, got {got}")]
    InsufficientData { min: usize, got: usize },

    #[error("target FMR {0} is not in (0, 1)")]
    InvalidTargetFmr(f64),

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),

    #[error("profile not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("activation conflict: another profile is already active for this cohort")]
    ActivationConflict,
}
