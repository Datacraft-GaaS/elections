use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A cosignature produced by a witness on an STH.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessCosignature {
    pub id: Uuid,
    pub sth_id: Uuid,
    pub witness_id: String,
    /// SPIFFE URI of the witness service.
    pub witness_spiffe_id: String,
    /// Ed25519 or HSM signature over the STH bytes (hex-encoded).
    pub signature: String,
    pub cosigned_at: DateTime<Utc>,
    /// Whether this was produced by an internal (IAB) or external witness.
    pub is_internal: bool,
}
