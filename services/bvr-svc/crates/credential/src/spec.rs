//! Typed models for `credential/openapi.yaml` — replaces `serde_json::Value`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mirrors `IssueRequest` schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRequest {
    pub credential_type: String,
    pub holder_id: Uuid,
    pub claims: serde_json::Value,
    pub valid_for_days: u32,
    pub batch_id: Option<Uuid>,
}

/// Mirrors `Credential` schema (subset required for contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub credential_id: Uuid,
    pub credential_type: String,
    pub holder_id: Uuid,
    pub holder_name: String,
    pub issuer_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
    pub claims: serde_json::Value,
    pub hsm_sig: String,
}

/// Mirrors `RevocationRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRequest {
    pub reason: String,
    pub revoking_officer_id: Uuid,
    pub effective_at: DateTime<Utc>,
}

/// Mirrors `CredentialProof`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialProof {
    pub credential_id: Uuid,
    pub proof_type: String,
    pub proof_value: String,
    pub verification_method: String,
    pub created_at: DateTime<Utc>,
    pub transparency_leaf_hash: Option<String>,
}

/// Mirrors `BatchSummary`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub batch_id: Uuid,
    pub credential_type: String,
    pub total_issued: i32,
    pub total_failed: i32,
    pub total_revoked: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
