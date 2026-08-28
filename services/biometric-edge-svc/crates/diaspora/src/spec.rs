//! Public contract handlers (diaspora-logistics/logistics.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};

use crate::api::SharedState;

fn page(items: &Value) -> Value {
    let n = items.as_array().map_or(0, |a| a.len());
    json!({ "items": items, "meta": { "page": 1, "page_size": n, "total": n, "total_pages": 1 } })
}

pub async fn list_embassies(State(s): State<SharedState>) -> impl IntoResponse {
    let guard = s.embassies.lock().expect("lock poisoned");
    let mut items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a["embassy_code"].as_str().cmp(&b["embassy_code"].as_str()));
    Json(page(&serde_json::to_value(items).unwrap()))
}

pub async fn get_embassy(State(s): State<SharedState>, Path(code): Path<String>) -> impl IntoResponse {
    match s.embassies.lock().expect("lock poisoned").get(&code) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "embassy not found" }))).into_response(),
    }
}

pub async fn create_ballot_dispatch(State(s): State<SharedState>, Json(req): Json<Value>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let v = json!({ "dispatch_id": id, "status": "dispatched", "created_at": chrono::Utc::now().to_rfc3339(), "request": req });
    s.ballot_dispatches.lock().expect("lock poisoned").insert(id, v.clone());
    (StatusCode::CREATED, Json(v))
}

pub async fn list_ballot_dispatches(State(s): State<SharedState>) -> impl IntoResponse {
    let guard = s.ballot_dispatches.lock().expect("lock poisoned");
    let mut items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a["dispatch_id"].as_str().cmp(&b["dispatch_id"].as_str()));
    Json(page(&serde_json::to_value(items).unwrap()))
}

pub async fn get_ballot_dispatch(State(s): State<SharedState>, Path(id): Path<String>) -> impl IntoResponse {
    match s.ballot_dispatches.lock().expect("lock poisoned").get(&id) {
        Some(v) => Json(v.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "dispatch not found" }))).into_response(),
    }
}

pub async fn confirm_ballot_delivery(State(s): State<SharedState>, Path(id): Path<String>, Json(req): Json<Value>) -> impl IntoResponse {
    let mut guard = s.ballot_dispatches.lock().expect("lock poisoned");
    match guard.get_mut(&id) {
        Some(v) => {
            v["status"] = json!("delivered");
            v["delivery_confirmation"] = req;
            Json(v.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "dispatch not found" }))).into_response(),
    }
}

pub async fn list_return_dispatches(State(s): State<SharedState>) -> impl IntoResponse {
    let guard = s.return_dispatches.lock().expect("lock poisoned");
    let mut items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a["return_id"].as_str().cmp(&b["return_id"].as_str()));
    Json(page(&serde_json::to_value(items).unwrap()))
}
