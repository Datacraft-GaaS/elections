//! HTTP server + public contract handlers for M39 ballot printing chain
//! (printing/chain.yaml).

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
    pub orders: Arc<Mutex<HashMap<String, Value>>>,
    pub deliveries: Arc<Mutex<Vec<Value>>>,
    pub audit: Arc<Mutex<Vec<Value>>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/printing/print-orders", get(list_print_orders).post(create_print_order))
        .route("/api/v1/printing/print-orders/{order_id}", get(get_print_order))
        .route("/api/v1/printing/print-orders/{order_id}/qc", post(submit_qc_decision))
        .route("/api/v1/printing/deliveries", post(record_ballot_delivery))
        .route("/api/v1/printing/audit-trail", get(get_printing_audit_trail))
        .with_state(state)
}

async fn create_print_order(
    State(state): State<SharedState>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let order_id = uuid::Uuid::now_v7().to_string();
    let order = json!({ "order_id": order_id, "status": "created", "request": req, "created_at": chrono::Utc::now().to_rfc3339() });
    state.orders.lock().expect("lock poisoned").insert(order_id.clone(), order.clone());
    state.audit.lock().expect("lock poisoned").push(json!({ "event": "order_created", "order_id": order_id, "at": chrono::Utc::now().to_rfc3339() }));
    (StatusCode::CREATED, Json(order))
}

async fn list_print_orders(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.orders.lock().expect("lock poisoned");
    let mut items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a["order_id"].as_str().cmp(&b["order_id"].as_str()));
    Json(json!({ "orders": items }))
}

async fn get_print_order(
    State(state): State<SharedState>,
    Path(order_id): Path<String>,
) -> impl IntoResponse {
    let guard = state.orders.lock().expect("lock poisoned");
    match guard.get(&order_id) {
        Some(o) => Json(o.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "order not found" }))).into_response(),
    }
}

async fn submit_qc_decision(
    State(state): State<SharedState>,
    Path(order_id): Path<String>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let mut guard = state.orders.lock().expect("lock poisoned");
    match guard.get_mut(&order_id) {
        Some(o) => {
            o["qc_decision"] = req.clone();
            o["status"] = json!("qc_reviewed");
            Json(o.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "order not found" }))).into_response(),
    }
}

async fn record_ballot_delivery(
    State(state): State<SharedState>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let delivery = json!({ "delivery_id": uuid::Uuid::now_v7().to_string(), "request": req, "recorded_at": chrono::Utc::now().to_rfc3339() });
    state.deliveries.lock().expect("lock poisoned").push(delivery.clone());
    state.audit.lock().expect("lock poisoned").push(json!({ "event": "delivery_recorded", "at": chrono::Utc::now().to_rfc3339() }));
    (StatusCode::CREATED, Json(delivery))
}

async fn get_printing_audit_trail(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.audit.lock().expect("lock poisoned");
    let items = guard.clone();
    drop(guard);
    Json(json!({ "audit_trail": items }))
}
