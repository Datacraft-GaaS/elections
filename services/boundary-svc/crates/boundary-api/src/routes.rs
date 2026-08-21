use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers;

/// Assemble the full Axum router for boundary-svc.
///
/// Public endpoints require no auth. Admin endpoints require a JWT with
/// SpiceDB boundary.admin permission (enforced by middleware, not yet wired
/// here; JWT extraction is a WP-2.2 concern).
pub fn router() -> Router {
    Router::new()
        // T2 public: from transparency.yaml
        .route(
            "/boundary/constituencies",
            get(handlers::list_constituencies),
        )
        .route(
            "/boundary/constituencies/{constituency_code}",
            get(handlers::get_constituency),
        )
        .route("/boundary/wards", get(handlers::list_wards))
        .route("/boundary/counties", get(handlers::list_counties))
        .route("/boundary/stations", get(handlers::list_stations))
        .route(
            "/boundary/stations/{station_code}",
            get(handlers::get_station),
        )
        .route("/boundary/changes", get(handlers::list_boundary_changes))
        // Admin
        .route(
            "/boundary/admin/publish",
            post(handlers::publish_boundary_snapshot),
        )
        // WP-2.4 brief additional paths
        .route("/api/v1/boundaries", get(handlers::list_boundaries))
        .route(
            "/api/v1/boundaries/bulk.geojson.gz",
            get(handlers::get_bulk_geojson),
        )
        .route("/api/v1/boundaries/{code}", get(handlers::get_boundary))
        .route(
            "/api/v1/boundaries/{code}/history",
            get(handlers::get_boundary_history),
        )
        .route(
            "/api/v1/geofence/station/{code}",
            get(handlers::get_geofence_station),
        )
        .route(
            "/api/v1/simulator/compute",
            get(handlers::simulator_compute),
        )
        .route(
            "/api/v1/metrics/fairness",
            get(handlers::get_fairness_metrics),
        )
        .route("/api/v1/metrics/impact", get(handlers::get_impact_metrics))
        .route("/api/v1/changes", get(handlers::get_changes))
        .route("/api/v1/submissions", post(handlers::create_submission))
        // Admin (JWT+SpiceDB)
        .route(
            "/api/v1/changes/{id}/approve",
            post(handlers::approve_change),
        )
        .route("/api/v1/geofence/upload", post(handlers::upload_geofence))
        .route(
            "/api/v1/imports/shapefile",
            post(handlers::import_shapefile),
        )
}
