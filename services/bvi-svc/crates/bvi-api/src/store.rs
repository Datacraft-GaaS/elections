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

//! Store abstraction: `BviStore` trait plus the Postgres implementation
//! (`PgStore`, runtime sqlx queries against the V1-V14 schema).
//!
//! The trait exists so the full axum router can be exercised hermetically in
//! unit tests against an in-memory fake; production always runs `PgStore`.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    Distress, DistressCreate, Identification, KitHealthPage, KitsHealthQuery, LookupAuditCtx,
    Session, SessionCreate, StationConfig, TelemetryCreate, VoterStation,
};
use bvi_turnout::{TurnoutScope, TurnoutSummary};

const IDEM_TTL_HOURS: i64 = 24;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("resource not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("voter already identified")]
    AlreadyIdentified,
    #[error("kit already revoked")]
    AlreadyRevoked,
    #[error("store unavailable")]
    Db(#[from] sqlx::Error),
}

/// Internal shape for a new identification event before it is persisted.
#[derive(Debug, Clone)]
pub struct IdentificationNew {
    pub event_id: Uuid,
    pub session_id: String,
    pub voter_id: String,
    pub biometric_ref: String,
    pub match_score: f64,
    /// Mirror-table outcome domain (matched / no_match / fallback_used / ambiguous).
    pub outcome: &'static str,
    pub identified_at: DateTime<Utc>,
    pub device_serial: String,
    pub station_code: String,
    pub election_id: String,
    pub modality: &'static str,
}

pub trait BviStore: Send + Sync + Clone + 'static {
    fn open_session(
        &self,
        req: &SessionCreate,
    ) -> impl std::future::Future<Output = Result<Session, StoreError>> + Send;
    fn get_session(
        &self,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<Session>, StoreError>> + Send;
    fn close_session(
        &self,
        session_id: &str,
        closed_at: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<Session, StoreError>> + Send;
    fn record_identification(
        &self,
        ev: &IdentificationNew,
    ) -> impl std::future::Future<Output = Result<Identification, StoreError>> + Send;
    fn insert_telemetry(
        &self,
        t: &TelemetryCreate,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
    fn insert_distress(
        &self,
        d: &DistressCreate,
    ) -> impl std::future::Future<Output = Result<Distress, StoreError>> + Send;
    fn station_config(
        &self,
        station_code: &str,
    ) -> impl std::future::Future<Output = Result<Option<StationConfig>, StoreError>> + Send;
    fn revoke_kit(
        &self,
        serial: &str,
        reason: &str,
        officer_id: &str,
    ) -> impl std::future::Future<Output = Result<RevokedKit, StoreError>> + Send;
    fn turnout_summary(
        &self,
        election_id: &str,
        scope: &TurnoutScope,
    ) -> impl std::future::Future<Output = Result<TurnoutSummary, StoreError>> + Send;
    fn kits_health(
        &self,
        q: &KitsHealthQuery,
    ) -> impl std::future::Future<Output = Result<KitHealthPage, StoreError>> + Send;
    fn voter_station(
        &self,
        voter_id: &str,
        election_id: &str,
        audit: &LookupAuditCtx,
    ) -> impl std::future::Future<Output = Result<Option<VoterStation>, StoreError>> + Send;
    fn idem_get(
        &self,
        operation: &str,
        key: &str,
    ) -> impl std::future::Future<Output = Result<Option<(u16, Option<Value>)>, StoreError>> + Send;
    fn idem_put(
        &self,
        operation: &str,
        key: &str,
        status: u16,
        body: Option<&Value>,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
}

/// Revoke response body (inline schema on revokeKit).
#[derive(Debug, Clone, serde::Serialize)]
pub struct RevokedKit {
    pub serial: String,
    pub revoked_at: DateTime<Utc>,
    pub reason: String,
}

/// Postgres-backed store. All queries are runtime strings (no `query!`).
#[derive(Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    fn is_unique_violation(err: &sqlx::Error) -> bool {
        err.as_database_error()
            .map(|d| d.is_unique_violation())
            .unwrap_or(false)
    }

    async fn load_session(&self, session_id: &str) -> sqlx::Result<Option<Session>> {
        let row = sqlx::query(
            r#"
            SELECT station_code, device_serial, election_id, status, opened_at, closed_at,
                   (SELECT COUNT(*) FROM identification_events e
                     WHERE e.session_id = b.session_id)::BIGINT AS ident_count
            FROM bvi_sessions b
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Session {
            session_id: session_id.to_owned(),
            station_code: r.get("station_code"),
            device_serial: r.get("device_serial"),
            election_id: r.get("election_id"),
            status: r.get("status"),
            opened_at: r.get("opened_at"),
            closed_at: r.get("closed_at"),
            identifications_count: r.get("ident_count"),
        }))
    }

    /// Insert an outbox row (V8 shape) inside the caller's transaction.
    async fn emit_outbox(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event_id: Uuid,
        topic: &str,
        partition_key: &str,
        payload: &Value,
    ) -> Result<(), StoreError> {
        sqlx::query(
            r#"
            INSERT INTO bvi_outbox (event_id, topic, partition_key, payload, headers)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(event_id)
        .bind(topic)
        .bind(partition_key)
        .bind(payload)
        .bind(Value::Object(Default::default()))
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }
}

impl BviStore for PgStore {
    async fn open_session(&self, req: &SessionCreate) -> Result<Session, StoreError> {
        let session_id = format!("ses-{}", Uuid::now_v7());
        let inserted = sqlx::query(
            r#"
            INSERT INTO bvi_sessions (session_id, station_code, device_serial, election_id)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(&session_id)
        .bind(&req.station_code)
        .bind(&req.device_serial)
        .bind(&req.election_id)
        .execute(&self.pool)
        .await;
        match inserted {
            Ok(_) => {}
            Err(e) if Self::is_unique_violation(&e) => {
                return Err(StoreError::Conflict(
                    "An active session already exists for this device and election day.".into(),
                ));
            }
            Err(e) => return Err(e.into()),
        }
        self.load_session(&session_id)
            .await?
            .ok_or(StoreError::NotFound)
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>, StoreError> {
        Ok(self.load_session(session_id).await?)
    }

    async fn close_session(
        &self,
        session_id: &str,
        closed_at: DateTime<Utc>,
    ) -> Result<Session, StoreError> {
        // Only an active session transitions; re-closing a closed or revoked
        // session is a no-op that returns the existing record (idempotent).
        sqlx::query(
            r#"
            UPDATE bvi_sessions SET status = 'closed', closed_at = $2
            WHERE session_id = $1 AND status = 'active'
            "#,
        )
        .bind(session_id)
        .bind(closed_at)
        .execute(&self.pool)
        .await?;
        self.load_session(session_id)
            .await?
            .ok_or(StoreError::NotFound)
    }

    async fn record_identification(
        &self,
        ev: &IdentificationNew,
    ) -> Result<Identification, StoreError> {
        let inserted = sqlx::query(
            r#"
            INSERT INTO identification_events
                (event_id, voter_id, station_code, kit_serial, modality, outcome,
                 ts_hlc, session_id, election_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(ev.event_id)
        .bind(&ev.voter_id)
        .bind(&ev.station_code)
        .bind(&ev.device_serial)
        .bind(ev.modality)
        .bind(ev.outcome)
        .bind(ev.identified_at.to_rfc3339())
        .bind(&ev.session_id)
        .bind(&ev.election_id)
        .execute(&self.pool)
        .await;
        match inserted {
            Ok(_) => {}
            Err(e) if Self::is_unique_violation(&e) => {
                // identification_events_voter_election_uq: one matched
                // identification per voter per election (EC-OPS-02).
                return Err(StoreError::AlreadyIdentified);
            }
            Err(e) => return Err(e.into()),
        }

        // Row-level audit trail (data-class B access). The shadow log carries
        // references only, never the biometric_ref token or raw identifiers.
        let audit_row = serde_json::json!({
            "event_id": ev.event_id,
            "session_id": ev.session_id,
            "station_code": ev.station_code,
            "kit_serial": ev.device_serial,
            "outcome": ev.outcome,
        });
        sqlx::query(
            r#"
            INSERT INTO audit.shadow_log (table_name, operation, new_row)
            VALUES ('identification_events', 'INSERT', $1)
            "#,
        )
        .bind(audit_row)
        .execute(&self.pool)
        .await?;

        Ok(Identification {
            identification_id: ev.event_id.to_string(),
            session_id: ev.session_id.clone(),
            voter_id: ev.voter_id.clone(),
            biometric_ref: ev.biometric_ref.clone(),
            match_score: ev.match_score,
            decision: crate::models::outcome_to_decision(ev.outcome).to_owned(),
            identified_at: ev.identified_at,
            device_serial: ev.device_serial.clone(),
        })
    }

    async fn insert_telemetry(&self, t: &TelemetryCreate) -> Result<(), StoreError> {
        sqlx::query(
            r#"
            INSERT INTO device_telemetry
                (device_serial, station_code, battery_pct, signal_strength_dbm,
                 temperature_c, ts)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&t.device_serial)
        .bind(&t.station_code)
        .bind(t.battery_pct)
        .bind(t.signal_strength_dbm)
        .bind(t.temperature_c)
        .bind(t.timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn insert_distress(&self, d: &DistressCreate) -> Result<Distress, StoreError> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO distress_events
                (station_code, device_serial, distress_type, details, raised_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(&d.station_code)
        .bind(&d.device_serial)
        .bind(&d.distress_type)
        .bind(&d.details)
        .bind(d.raised_at)
        .fetch_one(tx.as_mut())
        .await?;
        let distress_id: Uuid = row.get("id");

        // NOC delivery rides the transactional outbox so the alert survives a
        // Kafka outage (ADR-0015); Debezium publishes eel.bvi.distress_raised.v1.
        let payload = serde_json::json!({
            "distress_id": distress_id,
            "station_code": d.station_code,
            "device_serial": d.device_serial,
            "distress_type": d.distress_type,
            "raised_at": d.raised_at.to_rfc3339(),
        });
        Self::emit_outbox(
            &mut tx,
            Uuid::now_v7(),
            "eel.bvi.distress_raised.v1",
            &d.station_code,
            &payload,
        )
        .await?;
        tx.commit().await?;

        Ok(Distress {
            distress_id: distress_id.to_string(),
            station_code: d.station_code.clone(),
            device_serial: d.device_serial.clone(),
            distress_type: d.distress_type.clone(),
            details: d.details.clone(),
            raised_at: d.raised_at,
        })
    }

    async fn station_config(
        &self,
        station_code: &str,
    ) -> Result<Option<StationConfig>, StoreError> {
        let row = sqlx::query(
            r#"
            SELECT version, bundle_id, match_threshold, allowed_modalities,
                   timeout_s, fallback_allowed
            FROM kit_configs
            WHERE station_code = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(station_code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| {
            let modalities: Value = r.get("allowed_modalities");
            StationConfig {
                station_code: station_code.to_owned(),
                version: r.get("version"),
                bundle_id: r.get::<Uuid, _>("bundle_id").to_string(),
                match_threshold: r.get("match_threshold"),
                allowed_modalities: modalities
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
                timeout_s: r.get("timeout_s"),
                fallback_allowed: r.get("fallback_allowed"),
            }
        }))
    }

    async fn revoke_kit(
        &self,
        serial: &str,
        reason: &str,
        officer_id: &str,
    ) -> Result<RevokedKit, StoreError> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE kit_certs
            SET revoked = true, revoked_at = now(), revoked_reason = $2, revoked_by = $3
            WHERE kit_serial = $1 AND revoked = false
            RETURNING revoked_at
            "#,
        )
        .bind(serial)
        .bind(reason)
        .bind(officer_id)
        .fetch_optional(tx.as_mut())
        .await?;
        let Some(row) = row else {
            // No revocable cert: distinguish unknown kit from already revoked.
            let state = sqlx::query("SELECT revoked FROM kit_certs WHERE kit_serial = $1")
                .bind(serial)
                .fetch_optional(tx.as_mut())
                .await?;
            tx.rollback().await.ok();
            return match state {
                None => Err(StoreError::NotFound),
                Some(r) => {
                    if r.get::<bool, _>("revoked") {
                        Err(StoreError::AlreadyRevoked)
                    } else {
                        // Raced revoke; treat as already revoked.
                        Err(StoreError::AlreadyRevoked)
                    }
                }
            };
        };

        // Forcibly close any in-flight session from the revoked kit.
        sqlx::query(
            r#"
            UPDATE bvi_sessions SET status = 'revoked', closed_at = now()
            WHERE device_serial = $1 AND status = 'active'
            "#,
        )
        .bind(serial)
        .execute(tx.as_mut())
        .await?;

        // CRL distribution via SPIFFE consumes this event (kit_revoked.v1).
        let payload = serde_json::json!({
            "serial": serial,
            "reason": reason,
            "officer_id": officer_id,
        });
        Self::emit_outbox(
            &mut tx,
            Uuid::now_v7(),
            "eel.bvi.kit_revoked.v1",
            serial,
            &payload,
        )
        .await?;
        tx.commit().await?;

        Ok(RevokedKit {
            serial: serial.to_owned(),
            revoked_at: row.get("revoked_at"),
            reason: reason.to_owned(),
        })
    }

    async fn turnout_summary(
        &self,
        election_id: &str,
        scope: &TurnoutScope,
    ) -> Result<TurnoutSummary, StoreError> {
        Ok(bvi_turnout::turnout_summary(&self.pool, election_id, scope).await?)
    }

    async fn kits_health(&self, q: &KitsHealthQuery) -> Result<KitHealthPage, StoreError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (device_serial)
                       device_serial, station_code, battery_pct, ts
                FROM device_telemetry
                ORDER BY device_serial, ts DESC
            ),
            all_kits AS (
                SELECT kit_serial AS serial FROM kit_certs
                UNION
                SELECT DISTINCT device_serial FROM device_telemetry
            )
            SELECT k.serial,
                   l.station_code,
                   l.battery_pct,
                   l.ts AS last_seen,
                   COALESCE(kc.revoked, false) AS revoked,
                   (SELECT COUNT(*) FROM identification_events e
                     WHERE e.kit_serial = k.serial
                       AND e.received_at >= date_trunc('day', now() AT TIME ZONE 'utc')
                            AT TIME ZONE 'utc')::BIGINT AS ident_today,
                   (SELECT COUNT(*) FROM identification_events e
                     WHERE e.kit_serial = k.serial
                       AND e.received_at >= date_trunc('day', now() AT TIME ZONE 'utc')
                            AT TIME ZONE 'utc'
                       AND e.outcome = 'error')::BIGINT AS err_today
            FROM all_kits k
            LEFT JOIN latest l ON l.device_serial = k.serial
            LEFT JOIN kit_certs kc ON kc.kit_serial = k.serial
            WHERE ($1::text IS NULL OR l.station_code = $1)
            ORDER BY k.serial
            "#,
        )
        .bind(q.station_code.as_deref())
        .fetch_all(&self.pool)
        .await?;

        let now = Utc::now();
        let mut items: Vec<crate::models::KitHealth> = rows
            .into_iter()
            .map(|r| {
                let last_seen: Option<DateTime<Utc>> = r.get("last_seen");
                let online = last_seen
                    .map(|t| (now - t).num_seconds() <= 90)
                    .unwrap_or(false);
                crate::models::KitHealth {
                    serial: r.get("serial"),
                    station_code: r.get("station_code"),
                    battery_pct: r.get("battery_pct"),
                    online,
                    last_seen,
                    identification_count_today: r.get("ident_today"),
                    error_count_today: r.get("err_today"),
                }
            })
            .filter(|h| q.online.map(|want| h.online == want).unwrap_or(true))
            .collect();

        // Cursor pagination: opaque serial cursor, page beyond it.
        if let Some(cursor) = q.cursor.as_deref() {
            items.retain(|h| h.serial.as_str() > cursor);
        }
        let total = items.len() as i64;
        let page_size = q.effective_page_size() as usize;
        let next_cursor = if items.len() > page_size {
            items
                .get(page_size.saturating_sub(1))
                .map(|h| h.serial.clone())
        } else {
            None
        };
        items.truncate(page_size);
        let page_non_empty = !items.is_empty();

        Ok(KitHealthPage {
            items,
            total,
            next_cursor: next_cursor.filter(|_| page_non_empty),
        })
    }

    async fn voter_station(
        &self,
        voter_id: &str,
        election_id: &str,
        audit: &LookupAuditCtx,
    ) -> Result<Option<VoterStation>, StoreError> {
        let mut tx = self.pool.begin().await?;
        // Station assignment comes from the voter's latest matched
        // identification event; descriptive metadata must exist in the
        // polling_stations mirror or the lookup reports not_found rather
        // than inventing names.
        let row = sqlx::query(
            r#"
            SELECT ps.name, ps.constituency_code, ps.county_code, ev.station_code
            FROM (
                SELECT station_code FROM identification_events
                WHERE voter_id = $1 AND election_id = $2 AND outcome = 'matched'
                ORDER BY received_at DESC
                LIMIT 1
            ) ev
            JOIN polling_stations ps ON ps.station_code = ev.station_code
            "#,
        )
        .bind(voter_id)
        .bind(election_id)
        .fetch_optional(tx.as_mut())
        .await?;

        // FR-M07-020: every lookup is audited, found or not; rows are
        // non-erasable (FR-M07-021).
        sqlx::query(
            r#"
            INSERT INTO voter_lookup_audit
                (actor_session_hash, target_voter_id_hash, ip_hash, result_status)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(&audit.actor_session_hash)
        .bind(&sha256_hex(voter_id.as_bytes()))
        .bind(&audit.ip_hash)
        .bind(if row.is_some() { "found" } else { "not_found" })
        .execute(tx.as_mut())
        .await?;
        tx.commit().await?;

        Ok(row.map(|r| VoterStation {
            voter_id: voter_id.to_owned(),
            station_code: r.get("station_code"),
            polling_station_name: r.get("name"),
            constituency_code: r.get("constituency_code"),
            county_code: r.get("county_code"),
        }))
    }

    async fn idem_get(
        &self,
        operation: &str,
        key: &str,
    ) -> Result<Option<(u16, Option<Value>)>, StoreError> {
        // Lazy expiry sweep: the index on expires_at keeps this cheap.
        sqlx::query("DELETE FROM bvi_request_fingerprints WHERE expires_at < now()")
            .execute(&self.pool)
            .await?;
        let row = sqlx::query(
            r#"
            SELECT response_status, response_body
            FROM bvi_request_fingerprints
            WHERE operation = $1 AND idempotency_key = $2 AND expires_at >= now()
            "#,
        )
        .bind(operation)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| {
            (
                r.get::<i32, _>("response_status") as u16,
                r.get("response_body"),
            )
        }))
    }

    async fn idem_put(
        &self,
        operation: &str,
        key: &str,
        status: u16,
        body: Option<&Value>,
    ) -> Result<(), StoreError> {
        sqlx::query(
            r#"
            INSERT INTO bvi_request_fingerprints
                (operation, idempotency_key, response_status, response_body, expires_at)
            VALUES ($1, $2, $3, $4, now() + ($5 || ' hours')::interval)
            ON CONFLICT (operation, idempotency_key) DO NOTHING
            "#,
        )
        .bind(operation)
        .bind(key)
        .bind(i32::from(status))
        .bind(body.cloned())
        .bind(IDEM_TTL_HOURS.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(data);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}
