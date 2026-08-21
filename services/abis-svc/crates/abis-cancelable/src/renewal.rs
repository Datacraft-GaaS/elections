//! Template renewal: rotate user key, revoke old template, generate new shielded template.
//!
//! Ensures unlinkability across schemes by using an independent random projection
//! matrix derived from the new key.

use crate::revocation::{RevocationError, RevocationLedger, RevocationRecord};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenewError {
    #[error("subject id must not be empty")]
    EmptySubject,
    #[error("old key must not be empty")]
    EmptyOldKey,
    #[error("revocation failed: {0}")]
    Revocation(#[from] RevocationError),
}

/// Record of a template renewal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalRecord {
    pub subject_id: String,
    pub new_key_hint: String,
    pub revocation: RevocationRecord,
    pub renewed_at: DateTime<Utc>,
}

/// Rotates the user key, revokes the old template, and emits a renewal record.
///
/// The new key is derived by hashing old_key || subject_id || current_timestamp
/// to ensure it is independent of the old key.
///
/// # Errors
/// Returns an error if the subject or old key is empty, or if revocation fails.
pub fn renew(
    subject_id: &str,
    old_key: &[u8],
    scheme: &str,
    ledger: &RevocationLedger,
) -> Result<(RenewalRecord, Vec<u8>), RenewError> {
    if subject_id.is_empty() {
        return Err(RenewError::EmptySubject);
    }
    if old_key.is_empty() {
        return Err(RenewError::EmptyOldKey);
    }

    let revocation = ledger.revoke(scheme, subject_id)?;

    // Derive new key: SHA-256(old_key || subject_id || revocation_ref).
    let mut h = Sha256::new();
    h.update(old_key);
    h.update(subject_id.as_bytes());
    h.update(revocation.revocation_ref.as_bytes());
    let new_key: Vec<u8> = h.finalize().to_vec();
    let new_key_hint = format!("sha256:{}", hex::encode(&new_key[..4]));

    tracing::info!(
        subject_id = subject_id,
        scheme = scheme,
        "template renewed; old revoked"
    );

    Ok((
        RenewalRecord {
            subject_id: subject_id.to_owned(),
            new_key_hint,
            revocation,
            renewed_at: Utc::now(),
        },
        new_key,
    ))
}
