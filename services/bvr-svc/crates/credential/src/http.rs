//! Credential issuance public API (credential/openapi.yaml).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::spec::{IssueRequest, RevocationRequest};
use crate::store::{Store, StoreError};

/// Postgres-backed application state (no in-memory).
///
/// Production (`Store::Pg`) uses `PgPool` against `credential_api_*` tables
/// (`migrations/V054__credential_api.sql`). Hermetic tests use `Store::Mem`.
#[derive(Clone)]
pub struct AppState {
    pub store: Store,
}

impl Default for AppState {
    fn default() -> Self {
        Self { store: Store::mem() }
    }
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
    pub fn from_pool(pool: PgPool) -> SharedState {
        Arc::new(Self { store: Store::pg(pool) })
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

fn store_err(e: StoreError) -> axum::response::Response {
    match e {
        StoreError::NotFound => (StatusCode::NOT_FOUND, Json(json!({ "detail": "not found"}))).into_response(),
        StoreError::Db(err) => {
            tracing::error!(error = %err, "store db error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": "internal error"}))).into_response()
        }
    }
}

async fn issue_credential(State(s): State<SharedState>, Json(req): Json<IssueRequest>) -> impl IntoResponse {
    match s.store.issue_credential(req).await {
        Ok(cred) => (StatusCode::CREATED, Json(cred)).into_response(),
        Err(e) => store_err(e),
    }
}

async fn revoke_credential(
    State(s): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<RevocationRequest>,
) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid id" }))).into_response();
    };
    match s.store.revoke_credential(uid, req).await {
        Ok(Some(c)) => Json(c).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
        Err(e) => store_err(e),
    }
}

async fn get_credential(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid id" }))).into_response();
    };
    match s.store.get_credential(uid).await {
        Ok(Some(v)) => Json(v).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
        Err(e) => store_err(e),
    }
}

async fn get_credential_proof(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid id" }))).into_response();
    };
    match s.store.get_proof(uid).await {
        Ok(Some(p)) => Json(p).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
        Err(e) => store_err(e),
    }
}

async fn get_batch_summary(State(s): State<SharedState>, Path(batch_id): Path<String>) -> impl IntoResponse {
    let Ok(bid) = batch_id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid batch_id" }))).into_response();
    };
    match s.store.get_batch(bid).await {
        Ok(Some(v)) => Json(v).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "batch not found" }))).into_response(),
        Err(e) => store_err(e),
    }
}
