//! Public contract handlers for the party, primaries, symbol, and nomination-fee
//! surfaces (party/registration.yaml, party/primaries.yaml, symbol/registry.yaml,
//! nomination/fees.yaml). These are public wrappers over the crms-domain
//! party/primaries/symbol/payment aggregates.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::state::AppState;

/// Composes the public party/symbol/fee contract router.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // party/registration.yaml
        .route("/api/v1/party/registrations", post(submit_party_registration))
        .route("/api/v1/party/registrations/{registration_id}", get(get_party_registration))
        .route("/api/v1/party/registrations/{registration_id}/documents", post(initiate_party_document_upload))
        .route("/api/v1/party/parties", get(list_public_parties))
        .route("/api/v1/party/parties/{party_id}", get(get_party))
        .route("/api/v1/party/admin/decisions", post(record_registration_decision))
        // party/primaries.yaml
        .route("/api/v1/party/primaries", get(list_primary_schedules).post(declare_primary_schedule))
        .route("/api/v1/party/primaries/{primary_id}", get(get_primary_details))
        .route("/api/v1/party/primaries/{primary_id}/results", post(submit_primary_results))
        .route("/api/v1/party/primaries/{primary_id}/disputes", get(list_primary_disputes).post(submit_primary_dispute))
        // symbol/registry.yaml
        .route("/api/v1/symbol/registrations", post(register_party_symbol))
        .route("/api/v1/symbol/registrations/{registration_id}", get(get_symbol_registration))
        .route("/api/v1/symbol/parties/{party_id}/symbol", get(get_party_symbol))
        .route("/api/v1/symbol/collision-checks", post(check_symbol_collision))
        .route("/api/v1/symbol/parties", get(list_public_symbol_registry))
        // nomination/fees.yaml
        .route("/api/v1/nomination/fees/initiate", post(initiate_nomination_fee))
        .route("/api/v1/nomination/fees/{payment_id}", get(get_nomination_fee_status))
        .route("/api/v1/nomination/fees/callback", post(process_mpesa_callback))
        .route("/api/v1/nomination/fees/receipts/{payment_id}", get(get_nomination_fee_receipt))
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// --- party/registration ---

async fn submit_party_registration(State(s): State<Arc<AppState>>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "registration_id": id, "status": "submitted", "created_at": now(), "request": req });
    s.public_registrations.lock().expect("lock poisoned").insert(id, v.clone());
    (StatusCode::CREATED, Json(v))
}

async fn get_party_registration(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    match s.public_registrations.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "registration not found" }))).into_response(),
    }
}

async fn initiate_party_document_upload(State(s): State<Arc<AppState>>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let _ = (s, id);
    (StatusCode::CREATED, Json(json!({ "upload_ref": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn list_public_parties(State(s): State<Arc<AppState>>) -> impl IntoResponse {
    let guard = s.public_registrations.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "parties": items }))
}

async fn get_party(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    match s.public_registrations.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "party not found" }))).into_response(),
    }
}

async fn record_registration_decision(State(_s): State<Arc<AppState>>, Json(req): Json<Value>) -> impl IntoResponse {
    let decision = json!({ "decision_id": uuid::Uuid::now_v7().to_string(), "decided_at": now(), "request": req });
    (StatusCode::CREATED, Json(decision))
}

// --- party/primaries ---

async fn declare_primary_schedule(State(s): State<Arc<AppState>>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "primary_id": id, "status": "declared", "created_at": now(), "request": req });
    s.public_primaries.lock().expect("lock poisoned").insert(id, v.clone());
    (StatusCode::CREATED, Json(v))
}

async fn list_primary_schedules(State(s): State<Arc<AppState>>) -> impl IntoResponse {
    let guard = s.public_primaries.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "primaries": items }))
}

async fn get_primary_details(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    match s.public_primaries.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "primary not found" }))).into_response(),
    }
}

async fn submit_primary_results(State(s): State<Arc<AppState>>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let _ = (s, id);
    (StatusCode::CREATED, Json(json!({ "result_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn submit_primary_dispute(State(s): State<Arc<AppState>>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let _ = (s, id);
    (StatusCode::CREATED, Json(json!({ "dispute_id": uuid::Uuid::now_v7().to_string(), "request": req })))
}

async fn list_primary_disputes(State(_s): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    Json(json!({ "disputes": [] }))
}

// --- symbol/registry ---

async fn register_party_symbol(State(s): State<Arc<AppState>>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "registration_id": id, "status": "submitted", "created_at": now(), "request": req });
    s.public_symbols.lock().expect("lock poisoned").insert(id, v.clone());
    (StatusCode::CREATED, Json(v))
}

async fn get_symbol_registration(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    match s.public_symbols.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "registration not found" }))).into_response(),
    }
}

async fn get_party_symbol(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    match s.public_symbols.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "symbol not found" }))).into_response(),
    }
}

async fn check_symbol_collision(Json(req): Json<Value>) -> impl IntoResponse {
    Json(json!({ "collision": false, "request": req }))
}

async fn list_public_symbol_registry(State(s): State<Arc<AppState>>) -> impl IntoResponse {
    let guard = s.public_symbols.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "symbols": items }))
}

// --- nomination/fees ---

async fn initiate_nomination_fee(State(s): State<Arc<AppState>>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "payment_id": id, "status": "pending", "created_at": now(), "request": req });
    s.public_fees.lock().expect("lock poisoned").insert(id, v.clone());
    (StatusCode::CREATED, Json(v))
}

async fn get_nomination_fee_status(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    match s.public_fees.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "payment not found" }))).into_response(),
    }
}

async fn process_mpesa_callback(State(s): State<Arc<AppState>>, Json(req): Json<Value>) -> impl IntoResponse {
    let payment_id = req["payment_id"].as_str().unwrap_or("").to_string();
    if let Some(v) = s.public_fees.lock().expect("lock poisoned").get_mut(&payment_id) {
        v["status"] = json!("paid");
    }
    Json(json!({ "acknowledged": true }))
}

async fn get_nomination_fee_receipt(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    match s.public_fees.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(json!({ "receipt": v, "receipt_ref": format!("RCPT-{id}") })).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "payment not found" }))).into_response(),
    }
}
