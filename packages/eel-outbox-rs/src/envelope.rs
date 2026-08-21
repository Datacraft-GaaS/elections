//! Local envelope type for the outbox.
//!
//! This mirrors the fields from eel-events-rs::Envelope that the outbox
//! needs to write a row. Once eel-events-rs stabilises, callers can use
//! `OutboxEnvelope::from(&eel_events_rs::Envelope)`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All fields the outbox needs to persist an event row.
///
/// Build with [`OutboxEnvelope::builder`] or construct directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEnvelope {
    /// CloudEvents `id` (UUIDv7 string).
    pub id: String,
    /// CloudEvents `type` (e.g. `eel.rts.form34a_captured.v1`).
    pub event_type: String,
    /// CloudEvents `source` (SPIFFE URI of producer).
    pub source: String,
    /// CloudEvents `subject` (aggregate identifier).
    pub subject: Option<String>,
    /// CloudEvents `time` (RFC 3339).
    pub time: String,

    // eel_* extensions
    /// Stream identifier (aggregate root UUIDv7).
    pub eel_stream_id: String,
    /// Monotonic sequence within stream.
    pub eel_seq: i64,
    /// Hybrid logical clock value.
    pub eel_hlc: i64,
    /// SHA-256 hash of the preceding envelope in the stream.
    pub eel_prev_hash: Option<String>,
    /// Trace root UUIDv7.
    pub eel_correlation_id: Option<String>,
    /// Originating event UUIDv7.
    pub eel_causation_id: Option<String>,
    /// SPIFFE SVID fingerprint of signing key.
    pub eel_producer_key_id: Option<String>,
    /// Base64-encoded Ed25519 signature.
    pub eel_signature: Option<String>,
    /// Election identifier string.
    pub eel_election_id: Option<String>,
    /// PII data class (A/B/C/D/E per ADR-0016).
    pub eel_data_class: Option<String>,

    /// Event payload (must match registered Karapace schema).
    pub data: Value,

    /// Kafka topic to route this event to.
    pub topic: String,
    /// Kafka partition key (defaults to `eel_stream_id`).
    pub kafka_key: String,

    /// Aggregate type string (e.g. `form34a`).
    pub aggregate_type: String,
    /// Aggregate identifier (same as `eel_stream_id` for event-sourced roots).
    pub aggregate_id: String,
}

impl OutboxEnvelope {
    /// Serialize the full CloudEvents envelope to JSON bytes for size checking.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Return the Ed25519 signature bytes decoded from the base64 `eel_signature` field.
    pub fn signature_bytes(&self) -> Option<Vec<u8>> {
        let s = self.eel_signature.as_deref()?;
        let s = s.strip_prefix("ed25519:").unwrap_or(s);
        base64_decode(s).ok()
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    use std::collections::HashMap;
    // minimal decode — avoid pulling in the base64 crate separately
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let table: HashMap<u8, u8> = alphabet
        .bytes()
        .enumerate()
        .map(|(i, c)| (c, i as u8))
        .collect();
    let s = s.trim_end_matches('=');
    let mut out = Vec::with_capacity(s.len() * 3 / 4 + 1);
    let bytes: Vec<u8> = s.bytes().collect();
    for chunk in bytes.chunks(4) {
        let vals: Vec<u8> = chunk
            .iter()
            .map(|b| {
                *table
                    .get(b)
                    .ok_or_else(|| format!("invalid base64 char {b}"))
                    .unwrap()
            })
            .collect();
        match vals.len() {
            4 => {
                out.push((vals[0] << 2) | (vals[1] >> 4));
                out.push((vals[1] << 4) | (vals[2] >> 2));
                out.push((vals[2] << 6) | vals[3]);
            }
            3 => {
                out.push((vals[0] << 2) | (vals[1] >> 4));
                out.push((vals[1] << 4) | (vals[2] >> 2));
            }
            2 => {
                out.push((vals[0] << 2) | (vals[1] >> 4));
            }
            _ => {}
        }
    }
    Ok(out)
}
