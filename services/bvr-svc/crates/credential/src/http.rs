//! Credential issuance public API (credential/openapi.yaml).

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
    pub credentials: Arc<Mutex<HashMap<String, Value>>>,
    pub batches: Arc<Mutex<HashMap<String, Value>>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/credentials/issue", post(issue_credential))
        .route("/api/v1/credentials/{id}/revoke", post(revoke_credential))
        .route("/api/v1/credentials/{id}", get(get_credential))
        .route("/api/v1/credentials/{id}/proof", get(get_credential_proof))
        .route("/api/v1/credentials/batch/{batch_id}", get(get_batch_summary))
        .with_state(state)
}

async fn issue_credential(State(s): State<SharedState>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "credential_id": id, "status": "active", "issued_at": chrono::Utc::now().to_rfc3339(), "request": req });
    s.credentials.lock().expect("lock poisoned").insert(id, v.clone());
    (StatusCode::CREATED, Json(v))
}

async fn revoke_credential(State(s): State<SharedState>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let mut guard = s.credentials.lock().expect("lock poisoned");
    match guard.get_mut(&id) {
        Some(v) => {
            v["status"] = json!("revoked");
            v["revocation"] = req;
            Json(v.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
    }
}

async fn get_credential(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    match s.credentials.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
    }
}

async fn get_credential_proof(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    match s.credentials.lock().expect("lock poisoned").get(&id) {
        Some(_) => Json(json!({ "credential_id": id, "proof": uuid::Uuid::now_v7().to_string(), "verified": true })).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
    }
}

async fn get_batch_summary(State(s): State<SharedState>, Path(batch_id): Path<String>) -> impl IntoResponse {
    let guard = s.batches.lock().expect("lock poisoned");
    match guard.get(&batch_id) {
        Some(v) => Json(v.clone()).into_response(),
        None => Json(json!({ "batch_id": batch_id, "issued": 0, "revoked": 0, "status": "unknown" })).into_response(),
    }
}
