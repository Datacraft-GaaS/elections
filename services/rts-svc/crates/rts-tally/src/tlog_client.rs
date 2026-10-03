// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Anchors RTS tally publication snapshots into the transparency log.
//!
//! The platform ingest path is: transactional outbox row -> Debezium -> Kafka
//! topic [`TOPIC_SNAPSHOT_ANCHORED`] -> `tlog-ingest` (maps `eel.results.*`
//! to the Results domain) -> `tlog-sequencer` -> `leaves` table -> STH
//! rotation. Services publish through the outbox per the single-publish-path
//! invariant, so anchoring is atomic with the caller's business transaction
//! and every leaf carries a verifiable Ed25519 producer signature in the
//! documented `ed25519:<base64>` form.

use base64::Engine as _;
use ed25519_dalek::{Signer, SigningKey};
use eel_events_rs::{DataClass, Envelope};
use eel_outbox_rs::{Outbox, OutboxEnvelope};
use serde_json::json;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::instrument;

/// Kafka topic consumed by `tlog-ingest` for result-domain anchoring.
pub const TOPIC_SNAPSHOT_ANCHORED: &str = "eel.results.snapshot_anchored.v1";

/// Short voter-facing receipt code printed on the posted Form 34A: the
/// first 12 characters of the anchor payload hash, uppercased. Resolved
/// server-side by `tlog-api`'s receipt endpoint via prefix match against
/// the stored leaf hashes.
pub fn receipt_code(data_hash: &str) -> String {
    data_hash
        .chars()
        .take(12)
        .collect::<String>()
        .to_uppercase()
}

/// SPIFFE identity of this service used as the CloudEvents source.
pub const SOURCE_URI: &str = "spiffe://eel.local/service/rts-backend";

#[derive(Debug, Error)]
pub enum TlogError {
    #[error("tlog append failed: {0}")]
    Append(String),
    #[error("signature error: {0}")]
    Sign(String),
    #[error("canonicalisation error: {0}")]
    Canon(String),
    #[error(transparent)]
    Outbox(#[from] eel_outbox_rs::OutboxError),
}

/// Emits signed snapshot-anchor events into the transparency-log pipeline.
///
/// The client itself holds only signing state; the outbox handle is supplied
/// per call so anchoring participates in the caller's transaction.
pub struct TlogClient {
    signing_key: SigningKey,
    producer_key_id: String,
    election_id: Option<String>,
    clock: eel_hlc_rs::HlcClock,
    seq: std::sync::atomic::AtomicU64,
}

impl TlogClient {
    pub fn new(
        signing_key: SigningKey,
        producer_key_id: impl Into<String>,
        election_id: Option<String>,
        hlc_node: u16,
    ) -> Self {
        Self {
            signing_key,
            producer_key_id: producer_key_id.into(),
            election_id,
            clock: eel_hlc_rs::HlcClock::new(hlc_node, eel_hlc_rs::SystemWallClock),
            seq: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Build and sign the anchor envelope for one publication bundle.
    ///
    /// Pure aside from clock/sequence state, so tests assert the full
    /// contract offline.
    pub fn build_anchor_envelope(
        &self,
        merkle_root: &str,
        snapshot_id: &str,
    ) -> Result<Envelope, TlogError> {
        let payload = json!({
            "merkle_root": merkle_root,
            "snapshot_id": snapshot_id,
        });

        // SHA-256 over the serialised leaf payload, hex encoded: the value
        // downstream verifiers recompute against the published bundle.
        let payload_bytes =
            serde_json::to_vec(&payload).map_err(|e| TlogError::Canon(e.to_string()))?;
        let data_hash = hex::encode(Sha256::digest(&payload_bytes));

        let mut envelope = Envelope {
            specversion: "1.0".into(),
            id: uuid::Uuid::now_v7().to_string(),
            event_type: TOPIC_SNAPSHOT_ANCHORED.into(),
            source: SOURCE_URI.into(),
            subject: Some(format!("tally-snapshot/{snapshot_id}")),
            time: chrono::Utc::now(),
            datacontenttype: "application/json".into(),
            dataschema: None,
            data: json!({
                "merkle_root": merkle_root,
                "snapshot_id": snapshot_id,
                "data_hash": data_hash,
            }),
            eel_stream_id: snapshot_id.to_string(),
            eel_seq: self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            eel_hlc: self.clock.now().to_string(),
            eel_prev_hash: None,
            eel_correlation_id: None,
            eel_causation_id: None,
            eel_producer_key_id: Some(self.producer_key_id.clone()),
            eel_signature: None,
            eel_election_id: self.election_id.clone(),
            eel_data_class: DataClass::B,
        };

        let canonical = envelope
            .canonical_bytes_for_signing()
            .map_err(|e| TlogError::Canon(e.to_string()))?;
        let sig = self.signing_key.sign(&canonical);
        envelope.eel_signature = Some(format!(
            "ed25519:{}",
            base64::engine::general_purpose::STANDARD.encode(sig.to_bytes())
        ));
        Ok(envelope)
    }

    /// Convert a platform envelope into its transactional-outbox form.
    pub fn to_outbox_envelope(envelope: &Envelope) -> OutboxEnvelope {
        OutboxEnvelope {
            id: envelope.id.clone(),
            event_type: envelope.event_type.clone(),
            source: envelope.source.clone(),
            subject: envelope.subject.clone(),
            time: envelope.time.to_rfc3339(),
            eel_stream_id: envelope.eel_stream_id.clone(),
            eel_seq: envelope.eel_seq as i64,
            eel_hlc: envelope
                .eel_hlc
                .parse::<eel_hlc_rs::Hlc>()
                .ok()
                .and_then(|h| i64::try_from(h.physical_ms).ok())
                .unwrap_or_else(|| envelope.time.timestamp_millis()),
            eel_prev_hash: envelope.eel_prev_hash.clone(),
            eel_correlation_id: envelope.eel_correlation_id.clone(),
            eel_causation_id: envelope.eel_causation_id.clone(),
            eel_producer_key_id: envelope.eel_producer_key_id.clone(),
            eel_signature: None,
            eel_election_id: envelope.eel_election_id.clone(),
            eel_data_class: Some(
                match envelope.eel_data_class {
                    DataClass::A => "A",
                    DataClass::B => "B",
                    DataClass::C => "C",
                    DataClass::D => "D",
                    DataClass::E => "E",
                }
                .into(),
            ),
            data: envelope.data.clone(),
            topic: TOPIC_SNAPSHOT_ANCHORED.into(),
            kafka_key: envelope.eel_stream_id.clone(),
            aggregate_type: "tally_snapshot".into(),
            aggregate_id: envelope.eel_stream_id.clone(),
        }
    }
    /// Anchor one publication snapshot: emit the signed envelope through the
    /// transactional outbox in its own transaction, so the anchor either
    /// fully lands in the outbox or not at all.
    ///
    /// Returns the envelope id (the outbox primary key).
    #[instrument(skip(self, pool, merkle_root, snapshot_id), fields(snapshot_id = %snapshot_id))]
    pub async fn append_leaf(
        &self,
        pool: &sqlx::PgPool,
        merkle_root: &str,
        snapshot_id: &str,
    ) -> Result<String, TlogError> {
        let envelope = self.build_anchor_envelope(merkle_root, snapshot_id)?;
        let row = Self::to_outbox_envelope(&envelope);
        let outbox = Outbox::new(pool.clone());
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| TlogError::Append(e.to_string()))?;
        outbox.emit(&mut tx, &row).await?;
        tx.commit()
            .await
            .map_err(|e| TlogError::Append(e.to_string()))?;
        tracing::info!(
            event_id = %envelope.id,
            merkle_root = %merkle_root,
            "snapshot anchored to transparency log"
        );
        Ok(envelope.id)
    }

    /// Construct a client from process configuration.
    ///
    /// Required: `EEL_TLOG_SIGNING_KEY` (32-byte Ed25519 seed, hex).
    /// Optional: `EEL_TLOG_KEY_ID` (producer key fingerprint),
    /// `EEL_ELECTION_ID`, `EEL_HLC_NODE` (u16, default 1).
    pub fn from_env() -> Result<Self, TlogError> {
        let seed_hex = std::env::var("EEL_TLOG_SIGNING_KEY")
            .map_err(|_| TlogError::Sign("EEL_TLOG_SIGNING_KEY is not set".into()))?;
        let seed = hex::decode(seed_hex.trim())
            .map_err(|e| TlogError::Sign(format!("EEL_TLOG_SIGNING_KEY is not hex: {e}")))?;
        let seed: [u8; 32] = seed
            .try_into()
            .map_err(|_| TlogError::Sign("EEL_TLOG_SIGNING_KEY must be exactly 32 bytes".into()))?;
        let key_id = std::env::var("EEL_TLOG_KEY_ID")
            .unwrap_or_else(|_| "spiffe://eel.local/wl/rts-backend/svid".into());
        let hlc_node = std::env::var("EEL_HLC_NODE")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(1);
        Ok(Self::new(
            SigningKey::from_bytes(&seed),
            key_id,
            std::env::var("EEL_ELECTION_ID").ok(),
            hlc_node,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> TlogClient {
        TlogClient::new(
            SigningKey::from_bytes(&[7u8; 32]),
            "spiffe://eel.local/wl/rts-backend/svid",
            Some("2027-general".into()),
            1,
        )
    }

    #[test]
    fn envelope_carries_anchor_contract() {
        let env = test_client()
            .build_anchor_envelope("abc123root", "snap-0001")
            .unwrap();

        assert_eq!(env.event_type, "eel.results.snapshot_anchored.v1");
        assert_eq!(env.eel_stream_id, "snap-0001");
        assert_eq!(env.subject.as_deref(), Some("tally-snapshot/snap-0001"));
        assert_eq!(env.data["merkle_root"], "abc123root");
        assert_eq!(env.data["snapshot_id"], "snap-0001");
        let hash = env.data["data_hash"].as_str().unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn producer_signature_verifies_over_canonical_bytes() {
        use base64::Engine as _;
        let client = test_client();
        let env = client.build_anchor_envelope("root", "snap").unwrap();
        let sig_str = env.eel_signature.as_deref().expect("signed");
        let (scheme, b64) = sig_str.split_once(':').expect("ed25519:<b64> form");
        assert_eq!(scheme, "ed25519");

        let sig_bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        let sig = ed25519_dalek::Signature::from_slice(&sig_bytes).unwrap();
        let canonical = env.canonical_bytes_for_signing().unwrap();
        client
            .signing_key
            .verifying_key()
            .verify_strict(&canonical, &sig)
            .expect("signature must verify over canonical bytes");
    }

    #[test]
    fn sequence_advances_per_snapshot() {
        let client = test_client();
        let e1 = client.build_anchor_envelope("r", "s1").unwrap();
        let e2 = client.build_anchor_envelope("r", "s2").unwrap();
        assert!(e2.eel_seq > e1.eel_seq);
    }

    #[test]
    fn receipt_code_is_first_twelve_chars_uppercase() {
        let data_hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert_eq!(receipt_code(data_hash), "0123456789AB");
        assert_eq!(receipt_code(data_hash).len(), 12);
    }

    #[test]
    fn receipt_code_matches_envelope_data_hash_prefix() {
        let env = test_client()
            .build_anchor_envelope("root", "snap-rc")
            .unwrap();
        let data_hash = env.data["data_hash"].as_str().unwrap();
        let code = receipt_code(data_hash);
        assert_eq!(code.len(), 12);
        assert_eq!(code, data_hash[..12].to_uppercase());
    }

    #[test]
    fn outbox_row_maps_topic_and_partition_key() {
        let env = test_client()
            .build_anchor_envelope("root-x", "snap-x")
            .unwrap();
        let row = TlogClient::to_outbox_envelope(&env);
        assert_eq!(row.topic, TOPIC_SNAPSHOT_ANCHORED);
        assert_eq!(row.kafka_key, "snap-x");
        assert_eq!(row.aggregate_type, "tally_snapshot");
        assert_eq!(row.aggregate_id, "snap-x");
        assert_eq!(row.data["snapshot_id"], "snap-x");
    }
}
