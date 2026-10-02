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

//! Publishes custody Merkle roots into the transparency-log pipeline.
//!
//! The custody domain emits one signed-envelope row per window through the
//! transactional outbox on `eel.custody.merkle_root_published.v1`; tlog-ingest
//! maps that topic to the CustodyEvents domain and the sequencer anchors it.
//! The five-channel public fan-out (web, GitHub mirror, IPFS, Gazette print,
//! KBC radio) then happens in transparency-svc's publication stage, so this
//! crate only owns the single outbox write.

use chrono::{DateTime, Utc};
use eel_outbox_rs::{Outbox, OutboxEnvelope};
use serde_json::json;
use sqlx::PgPool;

/// SPIFFE identity of this service used as the CloudEvents source.
const SOURCE_URI: &str = "spiffe://eel.local/service/inventory-backend";

/// Kafka topic consumed by tlog-ingest for the CustodyEvents domain.
pub const TOPIC_MERKLE_ROOT_PUBLISHED: &str = "eel.custody.merkle_root_published.v1";

#[derive(Debug, thiserror::Error)]
/// MerkleError.
pub enum MerkleError {
    #[error("failed to publish merkle root: {0}")]
    /// variant.
    Publish(String),
    #[error(transparent)]
    /// variant.
    Outbox(#[from] eel_outbox_rs::OutboxError),
}

/// Publishes Merkle roots to the transparency-svc pipeline.
pub struct MerklePublisher;

impl MerklePublisher {
    /// new.
    pub fn new() -> Self {
        Self
    }

    /// Anchor a custody-window Merkle root: writes the outbox row in its own
    /// transaction and returns the event id.
    pub async fn publish_root(
        &self,
        pool: &PgPool,
        root_hash: &str,
        window_ts: DateTime<Utc>,
    ) -> Result<String, MerkleError> {
        let envelope = OutboxEnvelope {
            id: uuid::Uuid::now_v7().to_string(),
            event_type: TOPIC_MERKLE_ROOT_PUBLISHED.into(),
            source: SOURCE_URI.into(),
            subject: Some(format!("custody-window/{}", window_ts.to_rfc3339())),
            time: Utc::now().to_rfc3339(),
            eel_stream_id: format!("custody-merkle-{}", window_ts.timestamp()),
            eel_seq: 1,
            eel_hlc: Utc::now().timestamp_millis(),
            eel_prev_hash: None,
            eel_correlation_id: None,
            eel_causation_id: None,
            eel_producer_key_id: None,
            eel_signature: None,
            eel_election_id: None,
            eel_data_class: Some("C".into()),
            data: json!({
                "root_hash": root_hash,
                "window_ts": window_ts.to_rfc3339(),
            }),
            topic: TOPIC_MERKLE_ROOT_PUBLISHED.into(),
            kafka_key: root_hash.to_string(),
            aggregate_type: "custody_merkle_window".into(),
            aggregate_id: format!("custody-merkle-{}", window_ts.timestamp()),
        };

        let outbox = Outbox::new(pool.clone());
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| MerkleError::Publish(e.to_string()))?;
        outbox.emit(&mut tx, &envelope).await?;
        tx.commit()
            .await
            .map_err(|e| MerkleError::Publish(e.to_string()))?;

        tracing::info!(
            root_hash = %root_hash,
            event_id = %envelope.id,
            topic = %TOPIC_MERKLE_ROOT_PUBLISHED,
            "custody merkle root anchored"
        );
        Ok(envelope.id)
    }
}

impl Default for MerklePublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_maps_to_custody_domain() {
        // tlog-ingest topic_router maps any `eel.custody*` prefix to
        // Domain::CustodyEvents; keep the constant aligned with that contract.
        assert!(TOPIC_MERKLE_ROOT_PUBLISHED.starts_with("eel.custody"));
        assert!(TOPIC_MERKLE_ROOT_PUBLISHED.ends_with(".v1"));
    }
}
