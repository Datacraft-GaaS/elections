//! HTTP server + public contract handlers for M27 intent preservation
//! (support-intent/preservation.yaml).

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
    pub adjudications: Arc<Mutex<HashMap<String, Value>>>,
    pub alerts: Arc<Mutex<HashMap<String, Value>>>,
    pub toggle_audit: Arc<Mutex<Vec<Value>>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/v1/intent/premark", post(submit_pre_mark_validation))
        .route("/v1/intent/spoilt", post(submit_spoilt_ballot))
        .route(
            "/v1/intent/adjudications",
            get(list_adjudications).post(record_adjudication),
        )
        .route("/v1/intent/adjudications/{id}", get(get_adjudication))
        .route("/v1/intent/alerts", get(list_rejection_rate_alerts))
        .route(
            "/v1/intent/alerts/{id}/acknowledge",
            post(acknowledge_alert),
        )
        .route("/v1/intent/catalogue", get(list_adjudication_catalogue))
        .route("/v1/intent/toggle", get(list_toggle_audit).post(toggle_adjudication))
        .with_state(state)
}

async fn submit_pre_mark_validation(Json(req): Json<Value>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "valid": true, "request": req })))
}

async fn submit_spoilt_ballot(Json(req): Json<Value>) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(json!({ "spoilt_id": uuid::Uuid::now_v7().to_string(), "request": req })),
    )
}

async fn record_adjudication(
    State(state): State<SharedState>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let record = json!({ "adjudication_id": id, "request": req, "status": "open" });
    state
        .adjudications
        .lock()
        .expect("lock poisoned")
        .insert(id, record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn list_adjudications(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.adjudications.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "adjudications": items }))
}

async fn get_adjudication(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let guard = state.adjudications.lock().expect("lock poisoned");
    match guard.get(&id) {
        Some(a) => Json(a.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "adjudication not found" }))).into_response(),
    }
}

async fn list_rejection_rate_alerts(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.alerts.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "alerts": items }))
}

async fn acknowledge_alert(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut guard = state.alerts.lock().expect("lock poisoned");
    match guard.get_mut(&id) {
        Some(a) => {
            a["acknowledged"] = json!(true);
            Json(a.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "alert not found" }))).into_response(),
    }
}

async fn list_adjudication_catalogue() -> impl IntoResponse {
    Json(json!({ "catalogue": [] }))
}

async fn toggle_adjudication(
    State(state): State<SharedState>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let record = json!({ "toggle_id": uuid::Uuid::now_v7().to_string(), "request": req, "at": chrono::Utc::now().to_rfc3339() });
    state
        .toggle_audit
        .lock()
        .expect("lock poisoned")
        .push(record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn list_toggle_audit(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.toggle_audit.lock().expect("lock poisoned");
    let items = guard.clone();
    drop(guard);
    Json(json!({ "toggle_audit": items }))
}
