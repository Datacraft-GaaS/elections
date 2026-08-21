//! Error types for bulk operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BulkOpsError {
    #[error("operation not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("operation is not in a resumable state")]
    NotResumable,

    #[error("operation already completed or cancelled")]
    AlreadyTerminal,

    #[error("item processing failed: {0}")]
    ItemFailed(String),

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
