use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// --- Form 34A upload (multipart) -----------------------------------------------

/// Parsed multipart upload metadata (image bytes handled separately in handler).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Form34AUploadMeta {
    pub station_code: String,
    pub constituency_code: String,
    pub election_id: String,
    pub position_code: String,
    pub kit_serial: String,
    /// Ed25519 signature over (image_hash || station_code || election_id || timestamp_utc).
    pub kit_signature: String,
    /// Idempotency key (UUIDv7) preventing server-side re-execution.
    pub idempotency_key: Uuid,
    pub transmitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAccepted {
    pub form_id: Uuid,
    pub status: String,
    pub icr_job_id: Option<String>,
}

// --- SMS / USSD fallback codec payload -----------------------------------------

/// Wire-format for SMS fallback: serialised with CBOR canonical then compressed
/// with zstd. The `encoded_payload` field in HTTP is base64; the raw bytes are
/// the CBOR+zstd blob transmitted over SMS.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SmsFallbackHttpRequest {
    /// Base64-encoded CBOR+zstd payload (same bytes as transmitted over SMS).
    pub encoded_payload: String,
    pub kit_serial: String,
    pub sequence_number: u64,
    pub idempotency_key: Uuid,
}

/// Decoded, human-readable form used after decompression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsFallbackForm {
    /// Station code (alphanumeric, <= 12 bytes).
    pub station_code: String,
    pub election_id: String,
    pub position_code: String,
    pub votes_cast: u32,
    pub valid_votes: u32,
    pub rejected_votes: u32,
    pub spoilt_ballots: u32,
    pub candidate_totals: Vec<CandidateTotal>,
    pub kit_serial: String,
    pub sequence_number: u64,
    /// HLC timestamp (microseconds since epoch, UTC).
    pub transmitted_at_us: u64,
    /// Ed25519 signature bytes over canonical CBOR of the above fields.
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTotal {
    pub candidate_id: String,
    pub votes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SmsFallbackAccepted {
    pub form_id: Uuid,
    pub status: String,
}

// --- Transmission-attempt ledger -----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransmissionAttemptRequest {
    pub form_id: Uuid,
    pub kit_serial: String,
    pub channel: TransmissionChannel,
    pub status: AttemptStatus,
    pub idempotency_key: Uuid,
    pub attempted_at: String,
    pub error_detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransmissionChannel {
    PrimaryVpn,
    DualSimSwitchover,
    SmsFallback,
    UssdFallback,
    LoraMesh,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    Attempted,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransmissionAttemptAccepted {
    pub attempt_id: Uuid,
    pub status: String,
}

// --- Participation tokens -------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipationTokenResponse {
    pub station_code: String,
    /// Bounded random token (SHA-256 of (voter_ref || station_nonce)).
    pub token_sha256: String,
    pub issued_at: String,
    pub kit_serial: String,
}

// --- Dual-SIM idempotent resubmit ----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DualSimResubmitRequest {
    /// Same Idempotency-Key as the original attempt on SIM A.
    pub idempotency_key: Uuid,
    pub form_id: Uuid,
    pub kit_serial: String,
    pub sim_slot: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DualSimResubmitAccepted {
    pub form_id: Uuid,
    pub status: String,
    pub deduplicated: bool,
}
