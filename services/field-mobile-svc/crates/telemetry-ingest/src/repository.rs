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

//! sqlx runtime queries for telemetry tables.
//!
//! All queries use `sqlx::query()` at runtime; no compile-time DATABASE_URL required.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    AlertClass, ExitReason, MotionClass, Severity, SignalClass, TelemetryActivityFrame,
    TelemetryAlert, TelemetryDevice, TelemetryGeoDwell, TelemetrySession, TelemetrySignal,
    TelemetrySnapshot,
};
use crate::error::TelemetryError;

// ---------------------------------------------------------------------------
// TelemetryDevice
// ---------------------------------------------------------------------------

pub async fn insert_device(pool: &PgPool, device: &TelemetryDevice) -> Result<(), TelemetryError> {
    sqlx::query(
        r#"
        INSERT INTO eel_telemetry_devices
            (device_id, kit_id, app_id, persona_role, spiffe_id, hmac_key_hsm_ref,
             first_seen_at, last_seen_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(device.device_id)
    .bind(&device.kit_id)
    .bind(&device.app_id)
    .bind(&device.persona_role)
    .bind(&device.spiffe_id)
    .bind(&device.hmac_key_hsm_ref)
    .bind(device.first_seen_at)
    .bind(device.last_seen_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_device(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<Option<TelemetryDevice>, TelemetryError> {
    use sqlx::Row;
    let row = sqlx::query(
        r#"
        SELECT device_id, kit_id, app_id, persona_role, spiffe_id, hmac_key_hsm_ref,
               first_seen_at, last_seen_at, revoked_at, revocation_reason
        FROM eel_telemetry_devices WHERE device_id = $1
        "#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r: sqlx::postgres::PgRow| TelemetryDevice {
        device_id: r.get("device_id"),
        kit_id: r.get("kit_id"),
        app_id: r.get("app_id"),
        persona_role: r.get("persona_role"),
        spiffe_id: r.get("spiffe_id"),
        hmac_key_hsm_ref: r.get("hmac_key_hsm_ref"),
        first_seen_at: r.get("first_seen_at"),
        last_seen_at: r.get("last_seen_at"),
        revoked_at: r.get("revoked_at"),
        revocation_reason: r.get("revocation_reason"),
    }))
}

pub async fn revoke_device(
    pool: &PgPool,
    device_id: Uuid,
    reason: &str,
) -> Result<(), TelemetryError> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE eel_telemetry_devices SET revoked_at = $1, revocation_reason = $2 WHERE device_id = $3",
    )
    .bind(now)
    .bind(reason)
    .bind(device_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// TelemetrySession
// ---------------------------------------------------------------------------

pub async fn insert_session(
    pool: &PgPool,
    session: &TelemetrySession,
) -> Result<(), TelemetryError> {
    sqlx::query(
        r#"
        INSERT INTO eel_telemetry_sessions
            (session_id, device_id, actor_svid, entered_at, signals_count)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(session.session_id)
    .bind(session.device_id)
    .bind(&session.actor_svid)
    .bind(session.entered_at)
    .bind(session.signals_count)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn close_session(
    pool: &PgPool,
    session_id: Uuid,
    exit_reason: &ExitReason,
) -> Result<(), TelemetryError> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE eel_telemetry_sessions SET exited_at = $1, exit_reason = $2 WHERE session_id = $3",
    )
    .bind(now)
    .bind(exit_reason.as_str())
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_session(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<Option<TelemetrySession>, TelemetryError> {
    use sqlx::Row;
    let row = sqlx::query(
        r#"
        SELECT session_id, device_id, actor_svid, entered_at, exited_at, exit_reason, signals_count
        FROM eel_telemetry_sessions WHERE session_id = $1
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r: sqlx::postgres::PgRow| TelemetrySession {
        session_id: r.get("session_id"),
        device_id: r.get("device_id"),
        actor_svid: r.get("actor_svid"),
        entered_at: r.get("entered_at"),
        exited_at: r.get("exited_at"),
        exit_reason: r
            .get::<Option<String>, _>("exit_reason")
            .as_deref()
            .and_then(ExitReason::from_str),
        signals_count: r.get("signals_count"),
    }))
}

// ---------------------------------------------------------------------------
// TelemetrySignal
// ---------------------------------------------------------------------------

pub async fn insert_signal(pool: &PgPool, signal: &TelemetrySignal) -> Result<(), TelemetryError> {
    sqlx::query(
        r#"
        INSERT INTO eel_telemetry_signals
            (signal_id, device_id, session_id, class, payload, occurred_at, ingested_at,
             monotonic_counter, prev_hash, this_hash, hmac_signature, server_signature,
             trace_id, span_id, correlation_id, causation_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        "#,
    )
    .bind(signal.signal_id)
    .bind(signal.device_id)
    .bind(signal.session_id)
    .bind(signal.class.as_str())
    .bind(&signal.payload)
    .bind(signal.occurred_at)
    .bind(signal.ingested_at)
    .bind(signal.monotonic_counter)
    .bind(&signal.prev_hash)
    .bind(&signal.this_hash)
    .bind(&signal.hmac_signature)
    .bind(&signal.server_signature)
    .bind(&signal.trace_id)
    .bind(&signal.span_id)
    .bind(signal.correlation_id)
    .bind(signal.causation_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetch last signal hash and counter for a device (for chain verification on ingest).
pub async fn fetch_last_signal_state(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<Option<(i64, Vec<u8>)>, TelemetryError> {
    use sqlx::Row;
    let row = sqlx::query(
        r#"
        SELECT monotonic_counter, this_hash
        FROM eel_telemetry_signals
        WHERE device_id = $1
        ORDER BY monotonic_counter DESC
        LIMIT 1
        "#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r: sqlx::postgres::PgRow| {
        (
            r.get::<i64, _>("monotonic_counter"),
            r.get::<Vec<u8>, _>("this_hash"),
        )
    }))
}

/// Fetch all signal hashes for a device on a given date (for Merkle snapshot).
pub async fn fetch_signal_hashes_for_date(
    pool: &PgPool,
    device_id: Uuid,
    date: NaiveDate,
) -> Result<Vec<Vec<u8>>, TelemetryError> {
    use sqlx::Row;
    let rows = sqlx::query(
        r#"
        SELECT this_hash
        FROM eel_telemetry_signals
        WHERE device_id = $1
          AND occurred_at >= $2::date
          AND occurred_at < ($2::date + INTERVAL '1 day')
        ORDER BY monotonic_counter ASC
        "#,
    )
    .bind(device_id)
    .bind(date)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r: sqlx::postgres::PgRow| r.get::<Vec<u8>, _>("this_hash"))
        .collect())
}

// ---------------------------------------------------------------------------
// TelemetryActivityFrame
// ---------------------------------------------------------------------------

pub async fn insert_activity_frame(
    pool: &PgPool,
    frame: &TelemetryActivityFrame,
) -> Result<(), TelemetryError> {
    sqlx::query(
        r#"
        INSERT INTO eel_telemetry_activity_frames
            (frame_id, device_id, second_bucket, accel_x_avg, accel_y_avg, accel_z_avg,
             accel_max_g, gyro_x_avg, gyro_y_avg, gyro_z_avg, motion_class,
             battery_pct, battery_temp_c, network_class, lat_coarse, lon_coarse)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        "#,
    )
    .bind(frame.frame_id)
    .bind(frame.device_id)
    .bind(frame.second_bucket)
    .bind(frame.accel_x_avg)
    .bind(frame.accel_y_avg)
    .bind(frame.accel_z_avg)
    .bind(frame.accel_max_g)
    .bind(frame.gyro_x_avg)
    .bind(frame.gyro_y_avg)
    .bind(frame.gyro_z_avg)
    .bind(frame.motion_class.as_str())
    .bind(frame.battery_pct)
    .bind(frame.battery_temp_c)
    .bind(&frame.network_class)
    .bind(frame.lat_coarse)
    .bind(frame.lon_coarse)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// TelemetryGeoDwell
// ---------------------------------------------------------------------------

pub async fn insert_geo_dwell(
    pool: &PgPool,
    dwell: &TelemetryGeoDwell,
) -> Result<(), TelemetryError> {
    sqlx::query(
        r#"
        INSERT INTO eel_telemetry_geo_dwells
            (dwell_id, device_id, polygon_ref, entered_at, exited_at, dwell_seconds)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(dwell.dwell_id)
    .bind(dwell.device_id)
    .bind(&dwell.polygon_ref)
    .bind(dwell.entered_at)
    .bind(dwell.exited_at)
    .bind(dwell.dwell_seconds)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// TelemetryAlert
// ---------------------------------------------------------------------------

pub async fn insert_alert(pool: &PgPool, alert: &TelemetryAlert) -> Result<(), TelemetryError> {
    sqlx::query(
        r#"
        INSERT INTO eel_telemetry_alerts
            (alert_id, device_id, class, severity, detected_at, triggering_signal,
             payload, ecc_dispatched)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(alert.alert_id)
    .bind(alert.device_id)
    .bind(alert.class.as_str())
    .bind(alert.severity.as_str())
    .bind(alert.detected_at)
    .bind(alert.triggering_signal)
    .bind(&alert.payload)
    .bind(alert.ecc_dispatched)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn resolve_alert(
    pool: &PgPool,
    alert_id: Uuid,
    note: &str,
) -> Result<(), TelemetryError> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE eel_telemetry_alerts SET resolved_at = $1, resolution_note = $2 WHERE alert_id = $3",
    )
    .bind(now)
    .bind(note)
    .bind(alert_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// TelemetrySnapshot
// ---------------------------------------------------------------------------

pub async fn insert_snapshot(
    pool: &PgPool,
    snap: &TelemetrySnapshot,
) -> Result<(), TelemetryError> {
    sqlx::query(
        r#"
        INSERT INTO eel_telemetry_snapshots
            (snapshot_id, snapshot_date, device_id, device_merkle_root, snapshot_taken_at,
             tlog_leaf_index, tlog_root_hash, inclusion_proof)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (snapshot_date, device_id) DO UPDATE SET
            device_merkle_root = EXCLUDED.device_merkle_root,
            snapshot_taken_at  = EXCLUDED.snapshot_taken_at,
            tlog_leaf_index    = EXCLUDED.tlog_leaf_index,
            tlog_root_hash     = EXCLUDED.tlog_root_hash,
            inclusion_proof    = EXCLUDED.inclusion_proof
        "#,
    )
    .bind(snap.snapshot_id)
    .bind(snap.snapshot_date)
    .bind(snap.device_id)
    .bind(&snap.device_merkle_root)
    .bind(snap.snapshot_taken_at)
    .bind(snap.tlog_leaf_index)
    .bind(&snap.tlog_root_hash)
    .bind(&snap.inclusion_proof)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_snapshot(
    pool: &PgPool,
    snapshot_id: Uuid,
) -> Result<Option<TelemetrySnapshot>, TelemetryError> {
    use sqlx::Row;
    let row = sqlx::query(
        r#"
        SELECT snapshot_id, snapshot_date, device_id, device_merkle_root, snapshot_taken_at,
               tlog_leaf_index, tlog_root_hash, inclusion_proof
        FROM eel_telemetry_snapshots WHERE snapshot_id = $1
        "#,
    )
    .bind(snapshot_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r: sqlx::postgres::PgRow| TelemetrySnapshot {
        snapshot_id: r.get("snapshot_id"),
        snapshot_date: r.get("snapshot_date"),
        device_id: r.get("device_id"),
        device_merkle_root: r.get("device_merkle_root"),
        snapshot_taken_at: r.get("snapshot_taken_at"),
        tlog_leaf_index: r.get("tlog_leaf_index"),
        tlog_root_hash: r.get("tlog_root_hash"),
        inclusion_proof: r.get("inclusion_proof"),
    }))
}
