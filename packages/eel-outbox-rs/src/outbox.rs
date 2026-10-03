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

//! Transactional outbox implementation.
//!
//! `Outbox` is the sole path through which events reach Kafka. It inserts
//! an outbox row inside the caller's Postgres transaction; Debezium then
//! captures the row via CDC and publishes it to the configured Kafka topic.
//!
//! # Size guard
//!
//! If the serialised envelope exceeds 1 MiB, `emit` returns
//! `OutboxError::PayloadTooLarge`. Callers must upload large blobs to
//! MinIO via [`crate::minio::upload_blob`] and embed only the `BlobRef`
//! in the event `data` field.

use tracing::instrument;
use uuid::Uuid;

use crate::{envelope::OutboxEnvelope, error::OutboxError};

/// Maximum serialised envelope size in bytes (1 MiB).
pub const MAX_PAYLOAD_BYTES: usize = 1 << 20;

/// Transactional outbox.
///
/// Hold one per service; share via `Arc`. Callers pass a mutable reference
/// to an already-open `sqlx::Transaction` so the outbox row is committed
/// atomically with the business-logic write.
#[derive(Debug, Clone)]
pub struct Outbox {
    pool: sqlx::PgPool,
}

impl Outbox {
    /// Create an `Outbox` bound to the given connection pool.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Borrow the underlying pool (for migrations etc.).
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    /// Insert an event row into the `outbox` table inside `tx`.
    ///
    /// The row will be captured by Debezium and published to
    /// `envelope.topic`. The containing transaction must be committed by
    /// the caller; if it rolls back, the outbox row is discarded.
    ///
    /// # Errors
    ///
    /// - `OutboxError::PayloadTooLarge` if the serialised envelope exceeds 1 MiB.
    /// - `OutboxError::Sqlx` on any database error.
    #[instrument(skip(self, tx, envelope), fields(event_id = %envelope.id, topic = %envelope.topic))]
    pub async fn emit(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        envelope: &OutboxEnvelope,
    ) -> Result<(), OutboxError> {
        // Size guard: serialise the full envelope and check byte length.
        let payload_bytes = serde_json::to_vec(&envelope.data)?;
        if payload_bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(OutboxError::PayloadTooLarge {
                size: payload_bytes.len(),
            });
        }

        let id = Uuid::parse_str(&envelope.id)
            .map_err(|e| OutboxError::InvalidEnvelope(format!("invalid id UUID: {e}")))?;

        // Build headers jsonb from eel_* extensions.
        let headers = serde_json::json!({
            "specversion": "1.0",
            "type": envelope.event_type,
            "source": envelope.source,
            "subject": envelope.subject,
            "time": envelope.time,
            "eel_stream_id": envelope.eel_stream_id,
            "eel_seq": envelope.eel_seq,
            "eel_hlc": envelope.eel_hlc,
            "eel_prev_hash": envelope.eel_prev_hash,
            "eel_correlation_id": envelope.eel_correlation_id,
            "eel_causation_id": envelope.eel_causation_id,
            "eel_producer_key_id": envelope.eel_producer_key_id,
            "eel_election_id": envelope.eel_election_id,
            "eel_data_class": envelope.eel_data_class,
        });

        // Decode signature bytes (may be None for unsigned topics).
        let sig: Option<Vec<u8>> = envelope.signature_bytes();

        let payload_val = serde_json::to_value(&envelope.data)?;

        sqlx::query(
            r#"
            INSERT INTO outbox
                (id, aggregate_id, aggregate_type, topic, key,
                 payload, headers, sig, hlc)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(id)
        .bind(&envelope.aggregate_id)
        .bind(&envelope.aggregate_type)
        .bind(&envelope.topic)
        .bind(&envelope.kafka_key)
        .bind(sqlx::types::Json(&payload_val))
        .bind(sqlx::types::Json(&headers))
        .bind(sig)
        .bind(envelope.eel_hlc)
        .execute(tx.as_mut())
        .await?;

        tracing::debug!(
            event_id = %envelope.id,
            topic = %envelope.topic,
            aggregate_type = %envelope.aggregate_type,
            "outbox row inserted",
        );

        Ok(())
    }

    /// Run the outbox DDL migrations (idempotent).
    pub async fn migrate(&self) -> Result<(), OutboxError> {
        let mut conn = self.pool.acquire().await?;
        sqlx::raw_sql(crate::schema::CREATE_OUTBOX_TABLE)
            .execute(&mut *conn)
            .await?;
        sqlx::raw_sql(crate::schema::CREATE_OUTBOX_INDEX)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }
}
