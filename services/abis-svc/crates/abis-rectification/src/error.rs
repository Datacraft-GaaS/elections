//! Error types for rectification.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RectificationError {
    #[error("request not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: crate::domain::RectificationState,
        to: crate::domain::RectificationState,
    },

    #[error("evidence required: {0:?}")]
    EvidenceRequired(crate::domain::EvidenceType),

    #[error("ODPC reference required for template_removal requests")]
    OdpcRefRequired,

    #[error("officer assignment required for this request type")]
    OfficerRequired,

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
