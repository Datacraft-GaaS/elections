//! Telemetry query DSL types.
//!
//! `TelemetryQuery` is the canonical structured representation of an admin
//! telemetry query. It deserializes from a JSON body and is validated by
//! `TelemetryQueryValidator` before any database interaction.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Signal taxonomy (13 classes per spec section 2)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalClass {
    Location,
    Time,
    Activity,
    ProximalRfid,
    Movements,
    LoginLogout,
    Reads,
    Writes,
    Battery,
    LocksUnlocks,
    PhysicalDrops,
    Network,
    Attestation,
}

// ---------------------------------------------------------------------------
// Aggregation operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationOp {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Distinct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregation {
    pub op: AggregationOp,
    pub field: Field,
    pub alias: Option<String>,
}

// ---------------------------------------------------------------------------
// Group-by fields (allow-listed)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    DeviceId,
    OfficerId,
    SignalClass,
    Severity,
    County,
    StationId,
    Hour,
    Day,
    MotionClass,
    NetworkClass,
    BatteryBand,
    AttestationStatus,
}

// ---------------------------------------------------------------------------
// Geo-polygon (GeoJSON-compatible)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPolygon {
    /// Outer ring vertices: Vec<[lon, lat]>
    pub coordinates: Vec<[f64; 2]>,
}

// ---------------------------------------------------------------------------
// Time range
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Order-by direction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub field: Field,
    pub dir: OrderDir,
}

// ---------------------------------------------------------------------------
// Primary query struct
// ---------------------------------------------------------------------------

/// Canonical admin telemetry query.
///
/// All fields are optional except `time_range` which prevents full-table
/// scans. Defaults are conservative (limit=1000, no aggregations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryQuery {
    /// Restrict to specific device UUIDs. Empty means "all devices".
    #[serde(default)]
    pub device_ids: Vec<Uuid>,

    /// Signal classes to include. Empty means "all classes".
    #[serde(default)]
    pub signal_classes: Vec<SignalClass>,

    /// Mandatory time range.
    pub time_range: TimeRange,

    /// Optional spatial filter: only signals within this polygon.
    #[serde(default)]
    pub geo_polygon: Option<GeoPolygon>,

    /// Aggregation operations to apply.
    #[serde(default)]
    pub aggregations: Vec<Aggregation>,

    /// Group-by fields (only valid when aggregations are present).
    #[serde(default)]
    pub group_by: Vec<Field>,

    /// Row limit. Default 1000; hard cap 100000.
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Optional ordering.
    #[serde(default)]
    pub order_by: Vec<OrderBy>,
}

fn default_limit() -> usize {
    1_000
}

impl TelemetryQuery {
    /// Construct a minimal query for a single device over a time range.
    pub fn for_device(device_id: Uuid, time_range: TimeRange) -> Self {
        Self {
            device_ids: vec![device_id],
            signal_classes: vec![],
            time_range,
            geo_polygon: None,
            aggregations: vec![],
            group_by: vec![],
            limit: 1_000,
            order_by: vec![],
        }
    }
}
