//! Declaration domain event constants and payloads (WP-3.3, M51).
//! All events are CloudEvents 1.0 with Ed25519 signatures.

use serde::{Deserialize, Serialize};

// Event type constants
pub const EVENT_RESULT_DECLARED: &str = "eel.declaration.result_declared.v1";
pub const EVENT_RESULT_AMENDED: &str = "eel.declaration.result_amended.v1";
pub const EVENT_RECOUNT_ORDERED: &str = "eel.declaration.recount_ordered.v1";
pub const EVENT_CERTIFICATE_ISSUED: &str = "eel.declaration.certificate_issued.v1";
pub const EVENT_PETITION_WINDOW_OPENED: &str = "eel.declaration.petition_window_opened.v1";
pub const EVENT_PETITION_FILED: &str = "eel.declaration.petition_filed.v1";
pub const EVENT_JUDICIARY_ACK: &str = "eel.declaration.judiciary_ack.v1";
pub const EVENT_TIE_PENDING: &str = "eel.declaration.tie_pending.v1";
pub const EVENT_RUNOFF_REQUIRED: &str = "eel.declaration.runoff_created.v1";

// Consumed upstream
pub const EVENT_TALLY_PUBLISHED: &str = "eel.rts.tally_published.v1";

// ---- payloads ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultDeclaredPayload {
    /// UUIDv7 of the declaration aggregate
    pub declaration_id: String,
    pub contest_id: String,
    pub election_id: String,
    pub winner_id: Option<String>,
    pub declared_by: String,
    /// HLC timestamp (RFC3339)
    pub declared_at: String,
    /// HSM signing ceremony id for audit
    pub ceremony_id: String,
    /// Ed25519 signature from HSM countersign
    pub hsm_signature: String,
    /// Snapshot hash of the tally this declaration is derived from
    pub tally_snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultAmendedPayload {
    pub declaration_id: String,
    pub amendment_id: String,
    pub reason: String,
    pub amended_by: String,
    pub approved_by: String,
    pub amended_at: String,
    pub previous_snapshot_hash: String,
    pub new_snapshot_hash: String,
    /// Court order reference when triggered by recount (FR-M51-042)
    pub court_order_ref: Option<String>,
    pub new_tally_event_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecountOrderedPayload {
    pub declaration_id: String,
    pub court_order_ref: String,
    pub ordered_by: String,
    pub ordered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateIssuedPayload {
    pub certificate_id: String,
    pub declaration_id: String,
    pub winner_id: String,
    pub issued_at: String,
    pub hsm_signature: String,
    pub pdf_minio_ref: String,
    pub tlog_leaf_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetitionWindowOpenedPayload {
    pub petition_window_id: String,
    pub declaration_id: String,
    /// ISO8601 UTC
    pub window_open_at: String,
    /// T+28d 23:59:59 EAT inclusive (EC-TIM-04, EC-TIM-08)
    pub window_close_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetitionFiledPayload {
    pub petition_window_id: String,
    pub court_case_number: String,
    pub filed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudiciaryAckPayload {
    pub declaration_id: String,
    pub registrar_id: String,
    pub acknowledged_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiePendingPayload {
    pub declaration_id: String,
    pub contest_id: String,
    pub tied_candidates: Vec<String>,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunoffRequiredPayload {
    pub declaration_id: String,
    pub contest_id: String,
    /// Top-two candidate ids for the runoff
    pub runoff_candidates: Vec<String>,
    /// National vote percentage for each candidate
    pub national_votes: Vec<u64>,
    pub total_valid_votes: u64,
    /// Number of counties where candidate reached >=25% (Art. 138(4))
    pub counties_above_threshold: u32,
    pub occurred_at: String,
}
