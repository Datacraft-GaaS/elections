//! HTTP server + public contract handlers for M25 special-voter workflows
//! (support-special-voter/workflows.yaml).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

#[derive(Clone, Default)]
pub struct AppState {
    pub facilities: Arc<Mutex<HashMap<String, Value>>>,
    pub events: Arc<Mutex<HashMap<String, Vec<Value>>>>,
    pub forms: Arc<Mutex<HashMap<String, Value>>>,
    pub idp_reassignments: Arc<Mutex<Vec<Value>>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/v1/special/facilities", get(list_special_facilities))
        .route("/v1/special/facilities/{id}", get(get_special_facility))
        .route(
            "/v1/special/facilities/{id}/events",
            get(list_special_voter_events).post(submit_special_voter_event),
        )
        .route(
            "/v1/special/forms/34",
            get(list_special_forms34).post(submit_special_form34),
        )
        .route("/v1/special/forms/34/{id}", get(get_special_form34))
        .route(
            "/v1/special/idp/reassignments",
            post(record_idp_reassignment),
        )
        .route("/v1/special/dashboard", get(get_special_dashboard))
        .route(
            "/v1/special/audit/sample/{category}",
            get(get_audit_sample),
        )
        .route("/v1/special/public/counts", get(get_public_voter_counts))
        .with_state(state)
}

async fn list_special_facilities(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.facilities.lock().expect("lock poisoned");
    let mut items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    Json(json!({ "facilities": items }))
}

async fn get_special_facility(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let guard = state.facilities.lock().expect("lock poisoned");
    match guard.get(&id) {
        Some(f) => Json(f.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "facility not found" }))).into_response(),
    }
}

async fn submit_special_voter_event(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let record = json!({
        "facility_id": id.clone(),
        "event_type": req["event_type"],
        "officer_id": req["officer_id"],
        "recorded_at": chrono::Utc::now().to_rfc3339(),
    });
    state
        .events
        .lock()
        .expect("lock poisoned")
        .entry(id)
        .or_default()
        .push(record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn list_special_voter_events(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let guard = state.events.lock().expect("lock poisoned");
    let items = guard.get(&id).cloned().unwrap_or_default();
    drop(guard);
    Json(json!({ "events": items }))
}

async fn submit_special_form34(
    State(state): State<SharedState>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let id = req["id"].as_str().unwrap_or("").to_string();
    let mut form = req.clone();
    form["submitted_at"] = json!(chrono::Utc::now().to_rfc3339());
    state
        .forms
        .lock()
        .expect("lock poisoned")
        .insert(id.clone(), form.clone());
    (StatusCode::CREATED, Json(form))
}

async fn list_special_forms34(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.forms.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "forms": items }))
}

async fn get_special_form34(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let guard = state.forms.lock().expect("lock poisoned");
    match guard.get(&id) {
        Some(f) => Json(f.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "form not found" }))).into_response(),
    }
}

async fn record_idp_reassignment(
    State(state): State<SharedState>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let record = json!({
        "reassignment_id": uuid::Uuid::now_v7().to_string(),
        "voter_id_enc": req["voter_id_enc"],
        "idp_facility_id": req["idp_facility_id"],
        "recorded_at": chrono::Utc::now().to_rfc3339(),
    });
    state
        .idp_reassignments
        .lock()
        .expect("lock poisoned")
        .push(record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn get_special_dashboard(State(state): State<SharedState>) -> impl IntoResponse {
    let facilities = state.facilities.lock().expect("lock poisoned").len();
    Json(json!({
        "facilities_count": facilities,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn get_audit_sample(
    State(state): State<SharedState>,
    Path(category): Path<String>,
) -> impl IntoResponse {
    let guard = state.facilities.lock().expect("lock poisoned");
    let sample: Vec<Value> = guard
        .values()
        .filter(|f| f["category"].as_str().map_or(false, |c| c == category))
        .take(20)
        .cloned()
        .collect();
    drop(guard);
    Json(json!({ "category": category, "sample": sample }))
}

async fn get_public_voter_counts(State(state): State<SharedState>) -> impl IntoResponse {
    let facilities = state.facilities.lock().expect("lock poisoned").len();
    Json(json!({
        "total_facilities": facilities,
        "as_of": chrono::Utc::now().to_rfc3339(),
    }))
}
