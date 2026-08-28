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
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::spec::{BatchSummary, Credential, CredentialProof, IssueRequest, RevocationRequest};

/// In-memory store for credential contract handlers.
/// Swap boundary: replace `Mutex<HashMap>` with `sqlx::PgPool` + `Store` trait
/// (see `governance-svc/crates/rla/src/store.rs` pattern) when DB persistence lands.
/// Domain aggregates (`crms-domain`, `eel-events-rs`) already expect a `Store`.
#[derive(Clone, Default)]
pub struct AppState {
    pub credentials: Arc<Mutex<HashMap<Uuid, Credential>>>,
    pub batches: Arc<Mutex<HashMap<Uuid, BatchSummary>>>,
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

async fn issue_credential(State(s): State<SharedState>, Json(req): Json<IssueRequest>) -> impl IntoResponse {
    let now = Utc::now();
    let cred = Credential {
        credential_id: Uuid::now_v7(),
        credential_type: req.credential_type,
        holder_id: req.holder_id,
        holder_name: "holder".to_string(),
        issuer_id: "spiffe://eel.local/credential-issuer".to_string(),
        issued_at: now,
        expires_at: now + chrono::Duration::days(req.valid_for_days as i64),
        status: "active".to_string(),
        claims: req.claims,
        hsm_sig: String::new(),
    };
    let id = cred.credential_id;
    s.credentials.lock().expect("lock poisoned").insert(id, cred.clone());
    if let Some(bid) = req.batch_id {
        let mut b = s.batches.lock().expect("lock poisoned");
        b.entry(bid).or_insert_with(|| BatchSummary {
            batch_id: bid,
            credential_type: cred.credential_type.clone(),
            total_issued: 0,
            total_failed: 0,
            total_revoked: 0,
            status: "completed".to_string(),
            created_at: now,
            completed_at: Some(now),
        });
        if let Some(entry) = b.get_mut(&bid) {
            entry.total_issued += 1;
        }
    }
    (StatusCode::CREATED, Json(cred))
}

async fn revoke_credential(
    State(s): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<RevocationRequest>,
) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid id" }))).into_response();
    };
    let mut guard = s.credentials.lock().expect("lock poisoned");
    match guard.get_mut(&uid) {
        Some(v) => {
            v.status = "revoked".to_string();
            Json(v.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
    }
}

async fn get_credential(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid id" }))).into_response();
    };
    match s.credentials.lock().expect("lock poisoned").get(&uid) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
    }
}

async fn get_credential_proof(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid id" }))).into_response();
    };
    match s.credentials.lock().expect("lock poisoned").get(&uid) {
        Some(c) => Json(CredentialProof {
            credential_id: c.credential_id,
            proof_type: "EcdsaSecp384r1Signature2023".to_string(),
            proof_value: c.hsm_sig.clone(),
            verification_method: "https://keys.eel.local/credential-hsm#key-2026".to_string(),
            created_at: c.issued_at,
            transparency_leaf_hash: None,
        })
        .into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "credential not found" }))).into_response(),
    }
}

async fn get_batch_summary(State(s): State<SharedState>, Path(batch_id): Path<String>) -> impl IntoResponse {
    let Ok(bid) = batch_id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "detail": "invalid batch_id" }))).into_response();
    };
    let guard = s.batches.lock().expect("lock poisoned");
    match guard.get(&bid) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "batch not found" }))).into_response(),
    }
}
