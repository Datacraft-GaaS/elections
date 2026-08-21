use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Data classification per ADR-0016. A = most sensitive, E = public.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DataClass {
    A,
    B,
    C,
    D,
    E,
}

impl Default for DataClass {
    fn default() -> Self {
        Self::B
    }
}

/// CloudEvents 1.0 envelope with all `eel_*` platform extensions.
///
/// All fields follow the conventions in `docs/EVENT-CONVENTIONS.md`.
/// The shared library populates every extension; service code sets `data`,
/// `subject`, and optionally `eel_correlation_id` / `eel_causation_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// CloudEvents spec version. Always `"1.0"`.
    pub specversion: String,

    /// Globally unique event identifier (UUIDv7).
    pub id: String,

    /// Event type in `eel.<context>.<event_name>.v<major>` form.
    #[serde(rename = "type")]
    pub event_type: String,

    /// SPIFFE source URI of the producing service.
    pub source: String,

    /// Aggregate identifier (e.g. `form-34a/47-0001-001`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    /// NTP-authenticated wall-clock time. Legally authoritative.
    pub time: DateTime<Utc>,

    /// Always `"application/json"`.
    pub datacontenttype: String,

    /// JSON Schema URL for the payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataschema: Option<String>,

    /// Event payload matching the registered schema.
    pub data: serde_json::Value,

    // eel_* extensions ---------------------------------------------------------
    /// Aggregate root identifier (UUIDv7). Partition key on Kafka.
    pub eel_stream_id: String,

    /// Monotonic sequence number within the stream.
    pub eel_seq: u64,

    /// Hybrid Logical Clock timestamp (`physical_ms.counter.node`).
    pub eel_hlc: String,

    /// SHA-256 of the preceding event's canonical bytes within the stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eel_prev_hash: Option<String>,

    /// Trace root identifier. Propagated from upstream event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eel_correlation_id: Option<String>,

    /// `id` of the event that directly caused this one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eel_causation_id: Option<String>,

    /// SPIFFE SVID fingerprint of the signing key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eel_producer_key_id: Option<String>,

    /// Ed25519 signature over canonical JSON (minus this field). Format: `ed25519:<base64>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eel_signature: Option<String>,

    /// Election cycle identifier (e.g. `"2027-general"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eel_election_id: Option<String>,

    /// PII data classification of the payload (ADR-0016).
    pub eel_data_class: DataClass,
}

impl Envelope {
    /// Returns the canonical JSON bytes suitable for signature verification.
    ///
    /// Serializes the envelope to a `serde_json::Value`, removes the
    /// `eel_signature` field, then applies RFC 8785 canonicalization.
    pub fn canonical_bytes_for_signing(&self) -> Result<Vec<u8>, crate::error::EventError> {
        let mut value = serde_json::to_value(self)?;
        if let Some(obj) = value.as_object_mut() {
            obj.remove("eel_signature");
        }
        eel_crypto_rs::rfc8785::canonicalize(&value)
            .map_err(|e| crate::error::EventError::Canon(e.to_string()))
    }
}
