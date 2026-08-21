//! Error types for performance monitoring.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PerfMonitorError {
    #[error("insufficient holdout pairs: need >= {min}, got {got}")]
    InsufficientData { min: usize, got: usize },

    #[error("invalid threshold: must be in [0, 1), got {0}")]
    InvalidThreshold(f64),

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
