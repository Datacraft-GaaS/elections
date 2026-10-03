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

use chrono::Utc;
use eel_hlc_rs::Hlc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    aggregate::{Aggregate, AggregateSnapshot, StoredEvent},
    error::EventStoreError,
    schema::CREATE_SCHEMA_SQL,
};

/// Raw `eel_events` row as selected by the `stream_version, id, event_type,
/// payload, prev_hash, hlc, recorded_at` projections below.
type EventRow = (
    i64,
    Uuid,
    String,
    serde_json::Value,
    Option<String>,
    String,
    chrono::DateTime<Utc>,
);

/// Postgres-backed event store.
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Return the DDL that creates the event store tables.
    pub fn create_schema_sql() -> &'static str {
        CREATE_SCHEMA_SQL
    }

    /// Run the DDL against the pool (idempotent; uses `CREATE ... IF NOT EXISTS`).
    pub async fn migrate(&self) -> Result<(), EventStoreError> {
        sqlx::raw_sql(CREATE_SCHEMA_SQL).execute(&self.pool).await?;
        Ok(())
    }

    /// Append events to a stream, enforcing optimistic concurrency.
    ///
    /// `expected_version` must equal the current maximum `stream_version` for
    /// the stream (or -1 for a new stream). Returns the new maximum version.
    #[instrument(skip(self, events), fields(stream_id = %stream_id, expected = expected_version))]
    pub async fn append_events(
        &self,
        stream_id: Uuid,
        expected_version: i64,
        events: Vec<StoredEvent>,
    ) -> Result<i64, EventStoreError> {
        if events.is_empty() {
            return Ok(expected_version);
        }

        let mut tx = self.pool.begin().await?;

        // Lock the stream row and read current version.
        let current: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(stream_version) FROM eel_events WHERE stream_id = $1 FOR UPDATE",
        )
        .bind(stream_id)
        .fetch_one(&mut *tx)
        .await?;

        let actual = current.unwrap_or(-1);
        if actual != expected_version {
            return Err(EventStoreError::ConcurrencyConflict {
                expected: expected_version,
                actual,
            });
        }

        let mut version = expected_version;
        for ev in &events {
            version += 1;
            sqlx::query(
                r#"INSERT INTO eel_events
                   (stream_id, stream_version, id, event_type, payload, prev_hash, hlc, recorded_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            )
            .bind(stream_id)
            .bind(version)
            .bind(ev.id)
            .bind(&ev.event_type)
            .bind(&ev.payload)
            .bind(&ev.prev_hash)
            .bind(&ev.hlc)
            .bind(Utc::now())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(version)
    }

    /// Load events for a stream, optionally only those with HLC strictly after `after`.
    pub async fn load_events(
        &self,
        stream_id: Uuid,
        after: Option<Hlc>,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let rows: Vec<EventRow> = if let Some(hlc) = after {
            let hlc_str = hlc.to_string();
            sqlx::query_as(
                r#"SELECT stream_version, id, event_type, payload, prev_hash, hlc, recorded_at
                       FROM eel_events
                       WHERE stream_id = $1 AND hlc > $2
                       ORDER BY stream_version ASC"#,
            )
            .bind(stream_id)
            .bind(hlc_str)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"SELECT stream_version, id, event_type, payload, prev_hash, hlc, recorded_at
                       FROM eel_events
                       WHERE stream_id = $1
                       ORDER BY stream_version ASC"#,
            )
            .bind(stream_id)
            .fetch_all(&self.pool)
            .await?
        };

        let events = rows
            .into_iter()
            .map(
                |(stream_version, id, event_type, payload, prev_hash, hlc, recorded_at)| {
                    StoredEvent {
                        stream_id,
                        stream_version,
                        id,
                        event_type,
                        payload,
                        prev_hash,
                        hlc,
                        recorded_at,
                    }
                },
            )
            .collect();

        Ok(events)
    }

    /// Persist a snapshot, overwriting any prior snapshot for the same stream.
    pub async fn save_snapshot(&self, snap: AggregateSnapshot) -> Result<(), EventStoreError> {
        sqlx::query(
            r#"INSERT INTO eel_snapshots (stream_id, stream_version, state, taken_at)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (stream_id) DO UPDATE
               SET stream_version = EXCLUDED.stream_version,
                   state = EXCLUDED.state,
                   taken_at = EXCLUDED.taken_at"#,
        )
        .bind(snap.stream_id)
        .bind(snap.stream_version)
        .bind(snap.state)
        .bind(snap.taken_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load the latest snapshot for a stream, if one exists.
    pub async fn load_snapshot(
        &self,
        stream_id: Uuid,
    ) -> Result<Option<AggregateSnapshot>, EventStoreError> {
        let row: Option<(i64, serde_json::Value, chrono::DateTime<Utc>)> = sqlx::query_as(
            r#"SELECT stream_version, state, taken_at
               FROM eel_snapshots
               WHERE stream_id = $1"#,
        )
        .bind(stream_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            row.map(|(stream_version, state, taken_at)| AggregateSnapshot {
                stream_id,
                stream_version,
                state,
                taken_at,
            }),
        )
    }

    /// Replay all events into an aggregate, optionally starting from a snapshot.
    ///
    /// If a snapshot is found, it is passed to `restore_fn` to reconstruct the
    /// aggregate, then only events after the snapshot version are applied.
    pub async fn replay<A, F>(&self, stream_id: Uuid, restore_fn: F) -> Result<A, EventStoreError>
    where
        A: Aggregate + Default,
        F: FnOnce(AggregateSnapshot) -> A,
    {
        let snap = self.load_snapshot(stream_id).await?;
        if let Some(s) = snap {
            let snap_version = s.stream_version;
            let mut agg = restore_fn(s);
            let events = self
                .load_events_after_version(stream_id, snap_version)
                .await?;
            for ev in events {
                agg.apply(&ev)?;
            }
            return Ok(agg);
        }

        let mut aggregate = A::default();
        let events = self.load_events(stream_id, None).await?;
        for ev in events {
            aggregate.apply(&ev)?;
        }
        Ok(aggregate)
    }

    async fn load_events_after_version(
        &self,
        stream_id: Uuid,
        after_version: i64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let rows: Vec<EventRow> = sqlx::query_as(
            r#"SELECT stream_version, id, event_type, payload, prev_hash, hlc, recorded_at
                   FROM eel_events
                   WHERE stream_id = $1 AND stream_version > $2
                   ORDER BY stream_version ASC"#,
        )
        .bind(stream_id)
        .bind(after_version)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(stream_version, id, event_type, payload, prev_hash, hlc, recorded_at)| {
                    StoredEvent {
                        stream_id,
                        stream_version,
                        id,
                        event_type,
                        payload,
                        prev_hash,
                        hlc,
                        recorded_at,
                    }
                },
            )
            .collect())
    }
}
