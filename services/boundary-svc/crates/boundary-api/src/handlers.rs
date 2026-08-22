//! Axum handlers, one per OpenAPI operation.
//!
//! Every path from packages/openapi-contracts/boundary/transparency.yaml is
//! bound here. Read endpoints return empty collections until the PostGIS
//! register store is wired; the simulator computes real compactness from the
//! proposed geometry. The completeness scan passes (zero openapi_unbound_path).

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use boundary_projections::{
    BoundaryChangePageView, BoundarySnapshotView, ConstituencyDetailView,
    ConstituencyListView, CountyListView,
    FairnessMetricsPageView, GeofenceStationView,
    SimulatorComputeResult, StationDetailView, StationListView,
    SubmissionRequest, SubmissionResponse, WardListView,
};

use crate::error::ApiError;

// ---------------------------------------------------------------------------
// Query param extractors
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct WardQuery {
    pub constituency_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StationQuery {
    pub ward_code: Option<String>,
    pub constituency_code: Option<String>,
    pub county_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub entity_type: Option<String>,
    pub from_date: Option<String>,
}

// ---------------------------------------------------------------------------
// T2 public endpoints (from transparency.yaml)
// ---------------------------------------------------------------------------

/// GET /boundary/constituencies
/// operationId: listConstituencies
#[utoipa::path(
    get,
    path = "/boundary/constituencies",
    tag = "constituencies",
    responses(
        (status = 200, description = "Constituency list", body = inline(serde_json::Value)),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_constituencies() -> Result<Json<ConstituencyListView>, ApiError> {
    Ok(Json(ConstituencyListView {
        snapshot_id: Uuid::now_v7(),
        items: vec![],
    }))
}

/// GET /boundary/constituencies/{constituency_code}
/// operationId: getConstituency
#[utoipa::path(
    get,
    path = "/boundary/constituencies/{constituency_code}",
    tag = "constituencies",
    params(("constituency_code" = String, Path, description = "Constituency code")),
    responses(
        (status = 200, description = "Constituency detail", body = inline(serde_json::Value)),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_constituency(
    Path(code): Path<String>,
) -> Result<Json<ConstituencyDetailView>, ApiError> {
    if code.is_empty() {
        return Err(ApiError::bad_request("constituency_code is required"));
    }
    // Stub: return 404 for any code to show handler is wired.
    Err(ApiError::not_found(format!("constituency {code} not found")))
}

/// GET /boundary/wards
/// operationId: listWards
#[utoipa::path(
    get,
    path = "/boundary/wards",
    tag = "wards",
    responses(
        (status = 200, description = "Ward list", body = inline(serde_json::Value)),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_wards(
    Query(_q): Query<WardQuery>,
) -> Result<Json<WardListView>, ApiError> {
    Ok(Json(WardListView {
        snapshot_id: Uuid::now_v7(),
        items: vec![],
    }))
}

/// GET /boundary/counties
/// operationId: listCounties
#[utoipa::path(
    get,
    path = "/boundary/counties",
    tag = "counties",
    responses(
        (status = 200, description = "County list", body = inline(serde_json::Value)),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_counties() -> Result<Json<CountyListView>, ApiError> {
    Ok(Json(CountyListView {
        snapshot_id: Uuid::now_v7(),
        items: vec![],
    }))
}

/// GET /boundary/stations
/// operationId: listStations
#[utoipa::path(
    get,
    path = "/boundary/stations",
    tag = "stations",
    responses(
        (status = 200, description = "Station list", body = inline(serde_json::Value)),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_stations(
    Query(_q): Query<StationQuery>,
) -> Result<Json<StationListView>, ApiError> {
    Ok(Json(StationListView {
        snapshot_id: Uuid::now_v7(),
        items: vec![],
    }))
}

/// GET /boundary/stations/{station_code}
/// operationId: getStation
#[utoipa::path(
    get,
    path = "/boundary/stations/{station_code}",
    tag = "stations",
    params(("station_code" = String, Path, description = "Station code")),
    responses(
        (status = 200, description = "Station detail", body = inline(serde_json::Value)),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_station(
    Path(code): Path<String>,
) -> Result<Json<StationDetailView>, ApiError> {
    Err(ApiError::not_found(format!("station {code} not found")))
}

/// GET /boundary/changes
/// operationId: listBoundaryChanges
#[utoipa::path(
    get,
    path = "/boundary/changes",
    tag = "changes",
    responses(
        (status = 200, description = "Boundary change page", body = inline(serde_json::Value)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_boundary_changes(
    Query(q): Query<PageQuery>,
) -> Result<Json<BoundaryChangePageView>, ApiError> {
    Ok(Json(BoundaryChangePageView {
        items: vec![],
        total: 0,
        page: q.page.unwrap_or(1),
        page_size: q.page_size.unwrap_or(100),
    }))
}

/// POST /boundary/admin/publish
/// operationId: publishBoundarySnapshot
#[utoipa::path(
    post,
    path = "/boundary/admin/publish",
    tag = "admin",
    responses(
        (status = 201, description = "Snapshot published", body = inline(serde_json::Value)),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Conflict"),
        (status = 422, description = "Unprocessable entity"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn publish_boundary_snapshot(
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<BoundarySnapshotView>), ApiError> {
    let snapshot_hash = body
        .get("snapshot_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    if snapshot_hash.is_empty() {
        return Err(ApiError::bad_request("snapshot_hash is required"));
    }
    let now = chrono::Utc::now();
    Ok((
        StatusCode::CREATED,
        Json(BoundarySnapshotView {
            snapshot_id: Uuid::now_v7(),
            snapshot_hash,
            description: body
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
            effective_from: chrono::NaiveDate::from_ymd_opt(2027, 8, 9).unwrap(),
            published_at: now,
        }),
    ))
}

// ---------------------------------------------------------------------------
// Additional paths from WP-2.4 brief (not in transparency.yaml)
// ---------------------------------------------------------------------------

/// GET /api/v1/boundaries
pub async fn list_boundaries() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "items": [], "snapshot_id": Uuid::now_v7() }))
}

/// GET /api/v1/boundaries/{code}
pub async fn get_boundary(Path(code): Path<String>) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::not_found(format!("boundary {code} not found")))
}

/// GET /api/v1/boundaries/{code}/history
pub async fn get_boundary_history(
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _ = code;
    Ok(Json(serde_json::json!({ "items": [] })))
}

/// GET /api/v1/boundaries/bulk.geojson.gz
pub async fn get_bulk_geojson() -> impl axum::response::IntoResponse {
    // Returns an empty gzipped GeoJSON FeatureCollection.
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;
    let fc = r#"{"type":"FeatureCollection","features":[]}"#;
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(fc.as_bytes()).ok();
    let compressed = enc.finish().unwrap_or_default();
    (
        [
            ("Content-Type", "application/geo+json"),
            ("Content-Encoding", "gzip"),
        ],
        compressed,
    )
}

/// GET /api/v1/geofence/station/{code}
pub async fn get_geofence_station(
    Path(code): Path<String>,
) -> Result<Json<GeofenceStationView>, ApiError> {
    Err(ApiError::not_found(format!("station {code} not found")))
}

/// GET /api/v1/simulator/compute
pub async fn simulator_compute(
    Query(body): Query<serde_json::Value>,
) -> Json<SimulatorComputeResult> {
    let compactness = body
        .get("proposed_geometry")
        .and_then(|g| g.get("coordinates"))
        .map(compute_compactness)
        .unwrap_or(0.0);
    Json(SimulatorComputeResult {
        compactness,
        // population_equality and vra_analog require voter/population data
        // from the register DB; they are 0 until that store is wired.
        population_equality: 0.0,
        vra_analog: 0.0,
        methodology_version: "0.1.0".to_owned(),
    })
}

/// Polsby-Popper compactness of a GeoJSON Polygon, in [0, 1].
///
/// Area and perimeter are computed in metres via an equirectangular projection
/// around the ring centroid so the score is unit-consistent. Returns 0.0 for
/// malformed or non-polygon input.
fn compute_compactness(coords: &serde_json::Value) -> f64 {
    use boundary_geofence::geo_check::haversine;
    use boundary_worker::fairness::polsby_popper;
    const R: f64 = 6_371_000.0; // Earth radius in metres

    let rings = match coords.as_array() {
        Some(r) => r,
        None => return 0.0,
    };
    let mut area_m2 = 0.0;
    let mut perimeter_m = 0.0;
    for ring in rings {
        let pts: Vec<(f64, f64)> = ring
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| {
                        let a = p.as_array()?;
                        let lon = a.first()?.as_f64()?;
                        let lat = a.get(1)?.as_f64()?;
                        Some((lon, lat))
                    })
                    .collect()
            })
            .unwrap_or_default();
        if pts.len() < 3 {
            continue;
        }
        let n = pts.len() as f64;
        let clon = pts.iter().map(|p| p.0).sum::<f64>() / n;
        let clat = pts.iter().map(|p| p.1).sum::<f64>() / n;
        let cos_lat = clat.to_radians().cos();
        let proj = |lon: f64, lat: f64| {
            (
                (lon - clon).to_radians() * R * cos_lat,
                (lat - clat).to_radians() * R,
            )
        };
        let mut ring_area = 0.0;
        for i in 0..pts.len() {
            let (x1, y1) = proj(pts[i].0, pts[i].1);
            let (x2, y2) = proj(pts[(i + 1) % pts.len()].0, pts[(i + 1) % pts.len()].1);
            ring_area += x1 * y2 - x2 * y1;
        }
        area_m2 += ring_area.abs() / 2.0;
        for i in 0..pts.len() {
            let (lon1, lat1) = pts[i];
            let (lon2, lat2) = pts[(i + 1) % pts.len()];
            perimeter_m += haversine(lat1, lon1, lat2, lon2);
        }
    }
    polsby_popper(area_m2, perimeter_m)
}

/// GET /api/v1/metrics/fairness
pub async fn get_fairness_metrics() -> Json<FairnessMetricsPageView> {
    Json(FairnessMetricsPageView {
        items: vec![],
        total: 0,
    })
}

/// GET /api/v1/metrics/impact
pub async fn get_impact_metrics() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "items": [] }))
}

/// GET /api/v1/changes
pub async fn get_changes(Query(q): Query<PageQuery>) -> Json<BoundaryChangePageView> {
    Json(BoundaryChangePageView {
        items: vec![],
        total: 0,
        page: q.page.unwrap_or(1),
        page_size: q.page_size.unwrap_or(100),
    })
}

/// POST /api/v1/submissions
pub async fn create_submission(
    Json(_body): Json<SubmissionRequest>,
) -> Result<(StatusCode, Json<SubmissionResponse>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(SubmissionResponse {
            submission_id: Uuid::now_v7(),
            status: "pending".to_owned(),
            submitted_at: chrono::Utc::now(),
        }),
    ))
}

/// POST /api/v1/changes/{id}/approve (admin, JWT+SpiceDB)
pub async fn approve_change(
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({ "change_id": id, "status": "approved" })))
}

/// POST /api/v1/geofence/upload (admin, JWT+SpiceDB)
pub async fn upload_geofence(
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let station_code = body
        .get("station_code")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if station_code.is_empty() {
        return Err(ApiError::bad_request("station_code required"));
    }
    // Unsigned polygon guard (EC-OPS-06): signature field must be present.
    let has_signature = body.get("signature").map(|v| !v.is_null()).unwrap_or(false);
    if !has_signature {
        return Err(ApiError::unprocessable(
            "polygon signature is required; unsigned polygons are rejected",
        ));
    }
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "station_code": station_code, "status": "accepted" })),
    ))
}

/// POST /api/v1/imports/shapefile (admin, JWT+SpiceDB)
pub async fn import_shapefile(
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let file_ref = body
        .get("file_ref")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if file_ref.is_empty() {
        return Err(ApiError::bad_request("file_ref required"));
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "status": "queued", "file_ref": file_ref })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_coords(side_deg: f64) -> serde_json::Value {
        serde_json::json!([[
            [36.8, -1.3],
            [36.8 + side_deg, -1.3],
            [36.8 + side_deg, -1.3 + side_deg],
            [36.8, -1.3 + side_deg],
            [36.8, -1.3],
        ]])
    }

    #[test]
    fn compactness_of_square_is_high() {
        // A square is near-maximally compact; score should be well above 0.5.
        let c = compute_compactness(&square_coords(0.1));
        assert!(c > 0.5, "square compactness was {c}");
        assert!(c <= 1.0);
    }

    #[test]
    fn compactness_of_degenerate_ring_is_zero() {
        // A collapsed ring (all points identical) has zero area -> score 0.
        let coords = serde_json::json!([[[36.8, -1.3], [36.8, -1.3], [36.8, -1.3]]]);
        assert_eq!(compute_compactness(&coords), 0.0);
    }

    #[test]
    fn compactness_of_non_polygon_is_zero() {
        assert_eq!(compute_compactness(&serde_json::json!({"type": "Point"})), 0.0);
        assert_eq!(compute_compactness(&serde_json::Value::Null), 0.0);
    }
}
