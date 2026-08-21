//! Error types for model canary.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanaryError {
    #[error("canary not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("canary already active for model {0}")]
    AlreadyActive(uuid::Uuid),

    #[error("SLA breach: {0}")]
    SlaBreach(String),

    #[error("invalid traffic pct {0}: must be 0-100")]
    InvalidTrafficPct(u8),

    #[error("canary is not in InProgress state")]
    NotInProgress,

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
