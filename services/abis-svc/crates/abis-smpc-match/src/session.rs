//! SMPC session management.
//!
//! SmpcSession tracks party count, threshold, abort semantics, and timeout.
//! Writes eel_smpc_sessions row on creation (stubbed as tracing events).

use crate::protocol::{validate_party_count, Protocol, ProtocolError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("session {session_id} aborted: {reason}")]
    Aborted { session_id: String, reason: String },
    #[error("session timed out after {elapsed_ms}ms")]
    Timeout { elapsed_ms: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Pending,
    Active,
    Completed,
    Aborted,
}

/// An SMPC computation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmpcSession {
    pub session_id: String,
    pub protocol: Protocol,
    pub party_count: usize,
    pub threshold: usize,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub timeout_ms: u64,
}

impl SmpcSession {
    /// Creates a new session and logs a stub eel_smpc_sessions row.
    ///
    /// # Errors
    /// Returns `SessionError::Protocol` if the party count is below protocol minimum.
    pub fn create(
        protocol: Protocol,
        party_count: usize,
        threshold: usize,
        timeout_ms: u64,
    ) -> Result<Self, SessionError> {
        validate_party_count(&protocol, party_count)?;

        let session_id = Uuid::new_v4().to_string();
        let session = Self {
            session_id: session_id.clone(),
            protocol,
            party_count,
            threshold,
            status: SessionStatus::Pending,
            created_at: Utc::now(),
            timeout_ms,
        };

        tracing::info!(
            session_id = %session_id,
            party_count = party_count,
            threshold = threshold,
            event = "eel.abis.smpc_session.created.v1",
            "SMPC session created"
        );

        Ok(session)
    }

    /// Aborts the session with a reason.
    ///
    /// # Errors
    /// Returns `SessionError::Aborted`.
    pub fn abort(&mut self, reason: &str) -> Result<(), SessionError> {
        self.status = SessionStatus::Aborted;
        Err(SessionError::Aborted {
            session_id: self.session_id.clone(),
            reason: reason.to_owned(),
        })
    }

    /// Marks the session as completed.
    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
    }
}
