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

//! Async query executor.
//!
//! Runs a validated + planned `TelemetryQuery` against the PG read replica
//! and streams rows into a `Vec<serde_json::Value>`. Hard cap: 5M rows.

use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::dsl::{Field, SignalClass, TelemetryQuery};
use crate::planner::{CostModelPlanner, ScanOrder};

pub const ROW_CAP: usize = 5_000_000;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("row cap exceeded: query would return more than {ROW_CAP} rows")]
    RowCapExceeded,

    #[error("serialization error: {0}")]
    Serialization(String),
}

// ---------------------------------------------------------------------------
// Result row
// ---------------------------------------------------------------------------

pub type Row = serde_json::Value;

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

pub struct TelemetryExecutor {
    pool: PgPool,
}

impl TelemetryExecutor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Execute a validated + planned query. Returns up to `q.limit` rows,
    /// enforcing the absolute `ROW_CAP` before streaming to the caller.
    pub async fn execute(&self, q: &TelemetryQuery) -> Result<Vec<Row>, ExecutorError> {
        let plan = CostModelPlanner::plan(q);

        // Enforce 5M row cap on the estimate to avoid expensive queries
        if plan.estimated_rows > ROW_CAP as u64 {
            return Err(ExecutorError::RowCapExceeded);
        }

        let sql = build_sql(q, &plan.scan_order);
        let limit = q.limit.min(ROW_CAP) as i64;

        let rows = sqlx::query(&sql)
            .bind(q.time_range.from)
            .bind(q.time_range.to)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let mut result: Vec<Row> = Vec::with_capacity(rows.len());
        for row in rows {
            let json = row_to_json(&row)?;
            result.push(json);
        }

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// SQL builder
// ---------------------------------------------------------------------------

fn build_sql(q: &TelemetryQuery, scan_order: &ScanOrder) -> String {
    let mut parts: Vec<String> = vec![];

    parts.push(
        "SELECT signal_id, device_id, signal_class, occurred_at, payload, geo_lat, geo_lon, \
         severity, station_id, county, officer_id, motion_class, network_class, battery_pct, \
         attestation_status \
         FROM eel_telemetry_signals"
            .into(),
    );

    let mut wheres: Vec<String> = vec![];

    wheres.push("occurred_at >= $1".into());
    wheres.push("occurred_at < $2".into());

    if !q.device_ids.is_empty() {
        let ids: Vec<String> = q.device_ids.iter().map(|id| format!("'{}'", id)).collect();
        wheres.push(format!("device_id = ANY(ARRAY[{}]::uuid[])", ids.join(",")));
    }

    if !q.signal_classes.is_empty() {
        let classes: Vec<String> = q
            .signal_classes
            .iter()
            .map(|sc| format!("'{}'", signal_class_str(sc)))
            .collect();
        wheres.push(format!("signal_class = ANY(ARRAY[{}])", classes.join(",")));
    }

    if let Some(poly) = &q.geo_polygon {
        if !poly.coordinates.is_empty() {
            // Build a ST_Within check using a WKT polygon
            let points: Vec<String> = poly
                .coordinates
                .iter()
                .map(|[lon, lat]| format!("{} {}", lon, lat))
                .collect();
            let wkt = format!("POLYGON(({})) ", points.join(","));
            wheres.push(format!(
                "ST_Within(ST_Point(geo_lon, geo_lat), ST_GeomFromText('{}', 4326))",
                wkt
            ));
        }
    }

    if !wheres.is_empty() {
        parts.push(format!("WHERE {}", wheres.join(" AND ")));
    }

    // Ordering hint from planner
    let order_col = match scan_order {
        ScanOrder::DeviceThenTime => "device_id, occurred_at",
        ScanOrder::TimeThenDevice => "occurred_at, device_id",
        ScanOrder::PartitionScan => "occurred_at",
    };

    // Respect explicit order_by if provided
    if !q.order_by.is_empty() {
        let cols: Vec<String> = q
            .order_by
            .iter()
            .map(|ob| {
                format!(
                    "{} {}",
                    field_col(&ob.field),
                    match ob.dir {
                        crate::dsl::OrderDir::Asc => "ASC",
                        crate::dsl::OrderDir::Desc => "DESC",
                    }
                )
            })
            .collect();
        parts.push(format!("ORDER BY {}", cols.join(", ")));
    } else {
        parts.push(format!("ORDER BY {}", order_col));
    }

    parts.push("LIMIT $3".into());

    parts.join(" ")
}

fn signal_class_str(sc: &SignalClass) -> &'static str {
    match sc {
        SignalClass::Location => "location",
        SignalClass::Time => "time",
        SignalClass::Activity => "activity",
        SignalClass::ProximalRfid => "proximal_rfid",
        SignalClass::Movements => "movements",
        SignalClass::LoginLogout => "login_logout",
        SignalClass::Reads => "reads",
        SignalClass::Writes => "writes",
        SignalClass::Battery => "battery",
        SignalClass::LocksUnlocks => "locks_unlocks",
        SignalClass::PhysicalDrops => "physical_drops",
        SignalClass::Network => "network",
        SignalClass::Attestation => "attestation",
    }
}

fn field_col(f: &Field) -> &'static str {
    match f {
        Field::DeviceId => "device_id",
        Field::OfficerId => "officer_id",
        Field::SignalClass => "signal_class",
        Field::Severity => "severity",
        Field::County => "county",
        Field::StationId => "station_id",
        Field::Hour => "EXTRACT(HOUR FROM occurred_at)",
        Field::Day => "occurred_at::date",
        Field::MotionClass => "motion_class",
        Field::NetworkClass => "network_class",
        Field::BatteryBand => "FLOOR(battery_pct / 10) * 10",
        Field::AttestationStatus => "attestation_status",
    }
}

fn row_to_json(row: &sqlx::postgres::PgRow) -> Result<Row, ExecutorError> {
    use sqlx::Row as _;
    use std::collections::HashMap;

    let mut map: HashMap<String, serde_json::Value> = HashMap::new();

    let try_get_str = |col: &str| -> serde_json::Value {
        row.try_get::<String, _>(col)
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null)
    };

    for col in &[
        "signal_id",
        "device_id",
        "signal_class",
        "occurred_at",
        "severity",
        "station_id",
        "county",
        "officer_id",
        "motion_class",
        "network_class",
        "attestation_status",
    ] {
        map.insert((*col).to_string(), try_get_str(col));
    }

    for col in &["geo_lat", "geo_lon", "battery_pct"] {
        let val = row
            .try_get::<Option<f64>, _>(*col)
            .ok()
            .flatten()
            .map(|f| serde_json::json!(f))
            .unwrap_or(serde_json::Value::Null);
        map.insert((*col).to_string(), val);
    }

    if let Ok(payload) = row.try_get::<serde_json::Value, _>("payload") {
        map.insert("payload".into(), payload);
    }

    serde_json::to_value(map).map_err(|e| ExecutorError::Serialization(e.to_string()))
}
