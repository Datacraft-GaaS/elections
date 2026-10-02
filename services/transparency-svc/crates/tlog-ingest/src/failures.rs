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

//! Dead-letter table for events that could not be appended to the log.
//!
//! Integrity rule: a Kafka offset is only advanced past an event after that
//! event is durably in the `leaves` table (or is a duplicate, which the
//! sequencer deduplicates). Anything else lands here and blocks the offset,
//! so under-inclusion of results in the public log is impossible to miss.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Record a failed append. The caller must NOT advance the Kafka offset.
pub async fn record_failure(
    pool: &PgPool,
    topic: &str,
    partition: i32,
    offset: i64,
    event_id: Option<&str>,
    error_kind: &str,
    error_detail: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO tlog_ingest_failures
            (id, topic, partition, kafka_offset, event_id, error_kind, error_detail, failed_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, now())
        ON CONFLICT (topic, partition, kafka_offset) DO NOTHING
        "#,
    )
    .bind(topic)
    .bind(partition)
    .bind(offset)
    .bind(event_id)
    .bind(error_kind)
    .bind(error_detail)
    .execute(pool)
    .await?;
    Ok(())
}

/// One parked failure awaiting replay.
#[derive(Debug, sqlx::FromRow)]
pub struct IngestFailure {
    pub id: uuid::Uuid,
    pub topic: String,
    pub partition: i32,
    pub kafka_offset: i64,
    pub event_id: Option<String>,
    pub error_kind: String,
    pub error_detail: String,
    pub failed_at: DateTime<Utc>,
    pub replayed_at: Option<DateTime<Utc>>,
}

/// Failures not yet successfully appended.
pub async fn list_open(pool: &PgPool) -> Result<Vec<IngestFailure>, sqlx::Error> {
    sqlx::query_as::<_, IngestFailure>(
        "SELECT id, topic, partition, kafka_offset, event_id, error_kind, \
         error_detail, failed_at, replayed_at \
         FROM tlog_ingest_failures WHERE replayed_at IS NULL ORDER BY failed_at",
    )
    .fetch_all(pool)
    .await
}

/// Mark a failure as successfully replayed.
pub async fn mark_replayed(pool: &PgPool, id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tlog_ingest_failures SET replayed_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
