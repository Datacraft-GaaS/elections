//! Template revocation.
//!
//! revoke(scheme, subject) appends a revocation record to the in-memory ledger
//! and emits an eel.abis.template_revoked.v1 event.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RevocationError {
    #[error("subject id must not be empty")]
    EmptySubject,
    #[error("scheme must not be empty")]
    EmptyScheme,
    #[error("ledger lock poisoned")]
    LockPoisoned,
}

/// A record of a revoked template protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRecord {
    pub subject_id: String,
    pub scheme: String,
    pub revocation_ref: String,
    pub revoked_at: DateTime<Utc>,
}

/// In-memory ledger of revocation records.
///
/// Production implementation persists to eel_template_protections and
/// publishes eel.abis.template_revoked.v1 CloudEvents.
#[derive(Clone, Default)]
pub struct RevocationLedger {
    records: Arc<Mutex<Vec<RevocationRecord>>>,
}

impl RevocationLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a template revocation for the given subject and scheme.
    ///
    /// # Errors
    /// Returns an error if subject or scheme is empty.
    pub fn revoke(
        &self,
        scheme: &str,
        subject_id: &str,
    ) -> Result<RevocationRecord, RevocationError> {
        if subject_id.is_empty() {
            return Err(RevocationError::EmptySubject);
        }
        if scheme.is_empty() {
            return Err(RevocationError::EmptyScheme);
        }

        let revocation_ref = format!("rev-{}-{}", scheme, &subject_id[..subject_id.len().min(8)]);
        let record = RevocationRecord {
            subject_id: subject_id.to_owned(),
            scheme: scheme.to_owned(),
            revocation_ref,
            revoked_at: Utc::now(),
        };

        let mut ledger = self
            .records
            .lock()
            .map_err(|_| RevocationError::LockPoisoned)?;
        ledger.push(record.clone());

        tracing::info!(
            subject_id = subject_id,
            scheme = scheme,
            event = "eel.abis.template_revoked.v1",
            "template revoked"
        );

        Ok(record)
    }

    /// Returns all revocation records for a subject.
    pub fn records_for(&self, subject_id: &str) -> Result<Vec<RevocationRecord>, RevocationError> {
        let ledger = self
            .records
            .lock()
            .map_err(|_| RevocationError::LockPoisoned)?;
        Ok(ledger
            .iter()
            .filter(|r| r.subject_id == subject_id)
            .cloned()
            .collect())
    }

    /// Returns the total count of records in the ledger.
    pub fn len(&self) -> Result<usize, RevocationError> {
        let ledger = self
            .records
            .lock()
            .map_err(|_| RevocationError::LockPoisoned)?;
        Ok(ledger.len())
    }

    pub fn is_empty(&self) -> Result<bool, RevocationError> {
        Ok(self.len()? == 0)
    }
}
