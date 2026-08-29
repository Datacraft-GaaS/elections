//! Materialised read models for the public boundary delimitation view.
//!
//! These are the flat DTOs returned by the HTTP API. They are separate from
//! the domain aggregate so that the read path stays thin and allocation-cheap.
#![warn(missing_docs)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonGeometry {
    #[serde(rename = "type")]
    pub geom_type: String,
    pub coordinates: serde_json::Value,
}

impl GeoJsonGeometry {
    pub fn empty_polygon() -> Self {
        Self {
            geom_type: "Polygon".to_owned(),
            coordinates: serde_json::json!([[]]),
        }
    }
}

// ---------------------------------------------------------------------------
// Constituency read models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstituencyView {
    pub constituency_code: String,
    pub name: String,
    pub county_code: String,
    pub registered_stations: i32,
    pub geometry: GeoJsonGeometry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstituencyListView {
    pub snapshot_id: Uuid,
    pub items: Vec<ConstituencyView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstituencyDetailView {
    #[serde(flatten)]
    pub base: ConstituencyView,
    pub ward_codes: Vec<String>,
    pub area_km2: f32,
    pub effective_from: NaiveDate,
}

// ---------------------------------------------------------------------------
// Ward read models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardView {
    pub ward_code: String,
    pub name: String,
    pub constituency_code: String,
    pub county_code: String,
    pub registered_stations: i32,
    pub geometry: GeoJsonGeometry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardListView {
    pub snapshot_id: Uuid,
    pub items: Vec<WardView>,
}

// ---------------------------------------------------------------------------
// County read models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyView {
    pub county_code: String,
    pub name: String,
    pub constituency_count: i32,
    pub geometry: GeoJsonGeometry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyListView {
    pub snapshot_id: Uuid,
    pub items: Vec<CountyView>,
}

// ---------------------------------------------------------------------------
// Station read models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationView {
    pub station_code: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub ward_code: String,
    pub capacity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationListView {
    pub snapshot_id: Uuid,
    pub items: Vec<StationView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationDetailView {
    #[serde(flatten)]
    pub base: StationView,
    pub constituency_code: String,
    pub county_code: String,
    pub physical_address: String,
    pub accessibility_notes: Option<String>,
    pub effective_from: NaiveDate,
}

// ---------------------------------------------------------------------------
// Boundary change read models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryChangeView {
    pub change_id: Uuid,
    pub entity_type: String,
    pub entity_code: String,
    pub change_type: String,
    pub description: Option<String>,
    pub effective_from: NaiveDate,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub snapshot_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryChangePageView {
    pub items: Vec<BoundaryChangeView>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySnapshotView {
    pub snapshot_id: Uuid,
    pub snapshot_hash: String,
    pub description: String,
    pub effective_from: NaiveDate,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Fairness / impact
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessMetricView {
    pub id: Uuid,
    pub boundary_kind: String,
    pub metric_name: String,
    pub value: f64,
    pub computed_at: chrono::DateTime<chrono::Utc>,
    pub methodology_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessMetricsPageView {
    pub items: Vec<FairnessMetricView>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactView {
    pub station_code: String,
    pub old_constituency: Option<String>,
    pub new_constituency: Option<String>,
    pub population_affected: i64,
    pub compactness_delta: f64,
}

// ---------------------------------------------------------------------------
// Geofence station
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofenceStationView {
    pub station_code: String,
    pub polygon_version: String,
    pub geometry: GeoJsonGeometry,
    pub signed_by: String,
    pub effective_from: NaiveDate,
}

// ---------------------------------------------------------------------------
// Simulator
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorComputeRequest {
    pub proposed_geometry: GeoJsonGeometry,
    pub boundary_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorComputeResult {
    pub compactness: f64,
    pub population_equality: f64,
    pub vra_analog: f64,
    pub methodology_version: String,
}

// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRequest {
    pub submitter_class: String,
    pub payload_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionResponse {
    pub submission_id: Uuid,
    pub status: String,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}
