//! Mobile BFF public contract handlers (field-mobile-bff/bff.yaml + openapi.yaml).

use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::sse::ShardedBroadcast;

pub fn router() -> Router<Arc<ShardedBroadcast>> {
    Router::new()
        // field-mobile-bff/bff.yaml
        .route("/api/v1/mobile/bff/config", get(get_app_config))
        .route("/api/v1/mobile/bff/sessions", post(create_bff_session))
        .route("/api/v1/mobile/bff/sessions/refresh", post(refresh_bff_session))
        .route("/api/v1/mobile/bff/voter/verification", get(verify_voter))
        .route("/api/v1/mobile/bff/station/status", get(get_station_status))
        .route("/api/v1/mobile/bff/forms/sync", post(sync_forms))
        .route("/api/v1/mobile/bff/incidents", post(report_incident))
        .route("/api/v1/mobile/bff/dispatch/assignments", get(list_dispatch_assignments))
        .route("/api/v1/mobile/bff/dispatch/assignments/{assignment_id}", patch(update_dispatch_assignment))
        .route("/api/v1/mobile/bff/observer/access-pass", get(get_observer_access_pass))
        .route("/api/v1/mobile/bff/observer/observations", post(submit_observation))
        .route("/api/v1/mobile/bff/tamper-events", post(report_tamper_event))
        // field-mobile-bff/openapi.yaml
        .route("/v1/agent/check-in", post(agent_check_in))
        .route("/v1/agent/station/{code}/dashboard", get(get_station_dashboard))
        .route("/v1/agent/forms/34a/{id}", get(get_form34a))
        .route("/v1/agent/forms/34a/{id}/countersign", post(countersign_form34a))
        .route("/v1/agent/incidents", post(create_incident))
        .route("/v1/verify/registration", get(verify_registration))
}

async fn get_app_config() -> impl IntoResponse {
    Json(json!({ "config": {}, "version": "1.0.0" }))
}

async fn create_bff_session(Json(req): Json<Value>) -> impl IntoResponse {
    let _ = req;
    (StatusCode::CREATED, Json(json!({ "session_id": uuid::Uuid::now_v7().to_string(), "token": uuid::Uuid::now_v7().to_string() })))
}

async fn refresh_bff_session(Json(_req): Json<Value>) -> impl IntoResponse {
    Json(json!({ "token": uuid::Uuid::now_v7().to_string() }))
}

async fn verify_voter() -> impl IntoResponse {
    Json(json!({ "verified": false }))
}

async fn get_station_status() -> impl IntoResponse {
    Json(json!({ "status": "unknown" }))
}

async fn sync_forms(Json(req): Json<Value>) -> impl IntoResponse {
    Json(json!({ "synced": 0, "request": req }))
}

async fn report_incident(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "incident_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn list_dispatch_assignments() -> impl IntoResponse {
    Json(json!({ "assignments": [] }))
}

async fn update_dispatch_assignment(Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    Json(json!({ "assignment_id": id, "updated": true, "request": req }))
}

async fn get_observer_access_pass() -> impl IntoResponse {
    Json(json!({ "access_pass": null }))
}

async fn submit_observation(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "observation_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn report_tamper_event(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "tamper_event_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn agent_check_in(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "check_in_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn get_station_dashboard(Path(code): Path<String>) -> impl IntoResponse {
    Json(json!({ "station_code": code, "dashboard": {} }))
}

async fn get_form34a(Path(id): Path<String>) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(json!({ "detail": format!("form {id} not found") })))
}

async fn countersign_form34a(Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    Json(json!({ "form_id": id, "countersigned": true, "request": req }))
}

async fn create_incident(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "incident_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn verify_registration() -> impl IntoResponse {
    Json(json!({ "registered": false }))
}
