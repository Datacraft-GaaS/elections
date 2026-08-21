use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerificationQuery {
    pub query_id: String,
    pub channel: String,
    pub actor_session_hash: String,
    pub query_type: String,
    pub result_status: String,
    pub ts: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenVerifyRequest {
    pub token_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenVerifyResponse {
    pub query_id: String,
    pub token_sha256: String,
    pub included: bool,
    pub inclusion_proof_ref: Option<String>,
    pub verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipationTokenQuery {
    pub query_id: String,
    pub token_sha256: String,
    pub inclusion_proof_ref: Option<String>,
    pub verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GrievanceIntakeRequest {
    pub channel: String,
    pub description: String,
    pub station_code: Option<String>,
    pub photo_blob_id: Option<String>,
    pub audio_blob_id: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GrievanceRef {
    pub grievance_id: String,
    pub channel: String,
    pub submitted_at: DateTime<Utc>,
    pub ack_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CloneReport {
    pub report_id: String,
    pub app_id: String,
    pub package_name: String,
    pub reported_signature: String,
    pub platform: String,
    pub reported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CloneReportRequest {
    pub app_id: String,
    pub package_name: String,
    pub reported_signature: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BinaryIntegrityEvent {
    pub event_id: String,
    pub app_id: String,
    pub build_fingerprint: String,
    pub attestation_token: String,
    pub passed: bool,
    pub failure_reason: Option<String>,
    pub emitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BinaryIntegrityRequest {
    pub app_id: String,
    pub build_fingerprint: String,
    pub attestation_token: String,
}
