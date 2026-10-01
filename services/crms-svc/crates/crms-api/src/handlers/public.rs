//! Public contract handlers for the party, primaries, symbol, and nomination-fee
//! surfaces (party/registration.yaml, party/primaries.yaml, symbol/registry.yaml,
//! nomination/fees.yaml). These are public wrappers over the crms-domain
//! party/primaries/symbol/payment aggregates.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
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

async fn initiate_party_document_upload(
    State(s): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Response {
    // Real. Previously minted an upload_ref and discarded the request, so a
    // document upload could be initiated and never tracked.
    let exists = s
        .public_registrations
        .lock()
        .expect("lock poisoned")
        .contains_key(&id);
    if !exists {
        return not_found(format!("no party registration with id {id}"));
    }
    let upload_ref = uuid::Uuid::now_v7().to_string();
    let doc = json!({
        "upload_ref": upload_ref,
        "registration_id": id,
        "document_type": req.get("document_type").cloned().unwrap_or(Value::Null),
        "requested_at": now(),
    });
    s.documents
        .lock()
        .expect("lock poisoned")
        .insert(upload_ref.clone(), doc.clone());
    (StatusCode::CREATED, Json(doc)).into_response()
}

/// A problem+json response for a missing resource.
fn not_found(detail: String) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "type": "about:blank",
            "title": "Not Found",
            "status": 404,
            "detail": detail,
        })),
    )
        .into_response()
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

async fn record_registration_decision(State(s): State<Arc<AppState>>, Json(req): Json<Value>) -> Response {
    // Real. Previously minted a decision_id and discarded the decision, so a
    // registration could be approved or rejected with nothing recording it.
    let decision = json!({
        "decision_id": uuid::Uuid::now_v7().to_string(),
        "registration_id": req.get("registration_id").cloned().unwrap_or(Value::Null),
        "decision": req.get("decision").cloned().unwrap_or(Value::Null),
        "decided_at": now(),
    });
    s.registration_decisions
        .lock()
        .expect("lock poisoned")
        .insert(
            decision["decision_id"].as_str().unwrap_or_default().to_string(),
            decision.clone(),
        );
    (StatusCode::CREATED, Json(decision)).into_response()
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

async fn submit_primary_results(
    State(s): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Response {
    // Real. Previously minted a result_id and discarded the tally, so a primary
    // result could be submitted and never read back.
    let exists = s
        .public_primaries
        .lock()
        .expect("lock poisoned")
        .contains_key(&id);
    if !exists {
        return not_found(format!("no primary with id {id}"));
    }
    // Stored as the record that was submitted, keyed by its own id, so a
    // repeat submission cannot overwrite an earlier tally silently and the
    // declared `primary_id` is carried on the record.
    let submitted_at: chrono::DateTime<chrono::Utc> = now().parse().unwrap_or_else(|_| chrono::Utc::now());
    let result = json!({
        "result_id": uuid::Uuid::now_v7().to_string(),
        "primary_id": id,
        "results": req.get("results").cloned().unwrap_or(Value::Null),
        "submitted_at": submitted_at.to_rfc3339(),
    });
    let result_id: uuid::Uuid = result["result_id"]
        .as_str()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(uuid::Uuid::now_v7);
    s.submitted_primary_results
        .lock()
        .expect("lock poisoned")
        .insert(result_id, result.clone());
    (StatusCode::CREATED, Json(result)).into_response()
}

async fn submit_primary_dispute(
    State(s): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Response {
    // Real. Previously minted a dispute_id and discarded the dispute.
    let exists = s
        .public_primaries
        .lock()
        .expect("lock poisoned")
        .contains_key(&id);
    if !exists {
        return not_found(format!("no primary with id {id}"));
    }
    let dispute = json!({
        "dispute_id": uuid::Uuid::now_v7().to_string(),
        "primary_id": id,
        "grounds": req.get("grounds").cloned().unwrap_or(Value::Null),
        "status": "submitted",
        "submitted_at": now(),
    });
    s.primary_disputes
        .lock()
        .expect("lock poisoned")
        .insert(
            dispute["dispute_id"].as_str().unwrap_or_default().to_string(),
            dispute.clone(),
        );
    (StatusCode::CREATED, Json(dispute)).into_response()
}

async fn list_primary_disputes(State(s): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    // Real. Previously always an empty array, which reads as "no dispute has been
    // filed against this primary" rather than "nothing is stored".
    let disputes: Vec<Value> = s
        .primary_disputes
        .lock()
        .expect("lock poisoned")
        .values()
        .filter(|d| d["primary_id"].as_str() == Some(id.as_str()))
        .cloned()
        .collect();
    Json(json!({ "disputes": disputes }))
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

async fn check_symbol_collision(
    State(s): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Response {
    // Real. Previously answered `collision: false` for every request without
    // looking at the registered symbols, so a genuine collision with a symbol
    // already registered to another party was reported as clear.
    let proposed = req
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    let guard = s.public_symbols.lock().expect("lock poisoned");
    let conflicting = guard
        .values()
        .find(|v| {
            v["request"]["symbol"]
                .as_str()
                .map(|sym| sym.trim().eq_ignore_ascii_case(&proposed))
                .unwrap_or(false)
        })
        .map(|v| v["registration_id"].as_str().unwrap_or_default().to_string());
    drop(guard);

    if proposed.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "type": "about:blank",
                "title": "Unprocessable Entity",
                "status": 422,
                "detail": "a symbol must be supplied to check for a collision",
            })),
        )
            .into_response();
    }

    Json(json!({
        "symbol": proposed,
        "collision": conflicting.is_some(),
        "conflicting_registration_id": conflicting,
    }))
    .into_response()
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
