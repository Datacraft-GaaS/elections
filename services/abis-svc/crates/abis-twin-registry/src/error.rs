//! Error types for twin registry.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TwinRegistryError {
    #[error("twin group not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("subject {0} is already in a twin group")]
    AlreadyInGroup(uuid::Uuid),

    #[error("twin group must have at least 2 members")]
    InsufficientMembers,

    #[error("face-iris correlation check failed: {0}")]
    CorrelationCheckFailed(String),

    #[error("civil registry verification failed: {0}")]
    CivilRegistryFailed(String),

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
