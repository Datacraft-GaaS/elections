//! Error types for the adjudication crate.

use thiserror::Error;

use crate::audit::AuditError;
use crate::fsm::FsmError;

#[derive(Debug, Error)]
pub enum AdjudicationError {
    #[error("FSM: {0}")]
    Fsm(#[from] FsmError),

    #[error("audit chain: {0}")]
    Audit(#[from] AuditError),

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),

    #[error("case not found: {0}")]
    CaseNotFound(uuid::Uuid),

    #[error("no eligible reviewer found for tier {0:?}")]
    NoReviewerAvailable(crate::domain::ReviewerTier),
}
