use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// --- Queue -------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Form34AQueueItem {
    pub form_id: Uuid,
    pub station_code: String,
    pub constituency_code: String,
    pub election_id: String,
    pub position_code: String,
    pub status: Form34AStatus,
    pub icr_confidence: Option<f64>,
    pub received_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Form34AStatus {
    Pending,
    UnderReview,
    Verified,
    Disputed,
    Amended,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueueResponse {
    pub items: Vec<Form34AQueueItem>,
    pub total: usize,
}

// --- Verify ------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerifyForm34ARequest {
    pub ro_id: String,
    /// Ed25519 countersignature from the RO desktop HSM.
    pub ro_signature: String,
    pub idempotency_key: Uuid,
    pub verified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerifyAccepted {
    pub form_id: Uuid,
    pub event_id: Uuid,
    pub status: String,
}

// --- Amend -------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AmendForm34ARequest {
    pub ro_id: String,
    pub reason: String,
    pub amended_fields: serde_json::Value,
    pub ro_signature: String,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AmendAccepted {
    pub form_id: Uuid,
    pub amendment_id: Uuid,
    pub status: String,
}

// --- 34B compile -------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompileForm34BRequest {
    pub constituency_code: String,
    pub election_id: String,
    pub position_code: String,
    pub ro_id: String,
    /// Primary RO signature.
    pub ro_signature: String,
    /// Witness (second officer) signature for dual-officer requirement.
    pub witness_ro_id: String,
    pub witness_signature: String,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Form34BAccepted {
    pub form34b_id: Uuid,
    pub status: String,
    pub event_id: Uuid,
}

// --- 34C compile (CRO only) --------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompileForm34CRequest {
    pub region_code: String,
    pub election_id: String,
    pub position_code: String,
    pub cro_id: String,
    pub cro_signature: String,
    pub witness_cro_id: String,
    pub witness_signature: String,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Form34CAccepted {
    pub form34c_id: Uuid,
    pub status: String,
    pub event_id: Uuid,
}

// --- Disputes ----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisputeRequest {
    pub form_id: Uuid,
    pub raised_by: String,
    /// "ro" | "party_agent"
    pub raised_by_role: String,
    pub reason: String,
    /// Optional evidence references (MinIO object keys).
    pub evidence_refs: Vec<String>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisputeCreated {
    pub dispute_id: Uuid,
    pub form_id: Uuid,
    pub status: String,
    pub event_id: Uuid,
}

// --- Agent countersign -------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentCountersignRequest {
    pub form_id: Uuid,
    pub agent_id: String,
    pub party_code: String,
    pub agent_signature: String,
    pub idempotency_key: Uuid,
    pub signed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentCountersignAccepted {
    pub form_id: Uuid,
    pub event_id: Uuid,
    pub status: String,
}

// --- Session recording -------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionRecordingRequest {
    pub ro_id: String,
    pub action: SessionRecordingAction,
    /// SHA-256 hash of the current screen-capture segment (60 s anchor).
    pub segment_hash: Option<String>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionRecordingAction {
    Start,
    Anchor,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionRecordingAccepted {
    pub session_id: Uuid,
    pub event_id: Uuid,
    pub status: String,
    /// Transparency-log sequence number after anchor.
    pub tlog_seq: Option<u64>,
}
