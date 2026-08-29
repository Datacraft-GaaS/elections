//! HTTP server + public contract handlers for M39 ballot printing chain
//! (printing/chain.yaml).

use std::sync::Arc;

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

use crate::spec::{AuditEvent, DeliveryRecord, DeliveryRecordRequest, PrintOrder, PrintOrderRequest, QCDecision};

/// In-memory store for printing contract handlers.
/// Swap boundary: replace `SharedMap` with `sqlx::PgPool` + `Store` trait
/// (see `governance-svc/crates/rla/src/store.rs` pattern) when persistence lands.
/// Uses `eel-spec-helpers-rs::SharedMap` to deduplicate `Arc<Mutex<HashMap>>` boilerplate (TD-03/TD-08).
#[derive(Clone, Default)]
pub struct AppState {
    /// orders.
    pub orders: eel_spec_helpers_rs::SharedMap<Uuid, PrintOrder>,
    /// deliveries.
    pub deliveries: eel_spec_helpers_rs::SharedMap<Uuid, DeliveryRecord>,
    /// audit.
    pub audit: Arc<std::sync::Mutex<Vec<AuditEvent>>>,
}

/// SharedState.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// new_shared.
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
}

/// router.
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
    Json(req): Json<PrintOrderRequest>,
) -> impl IntoResponse {
    let now = Utc::now();
    let order = PrintOrder {
        order_id: Uuid::now_v7(),
        election_id: req.election_id,
        position_code: req.position_code,
        constituency_code: req.constituency_code,
        ballot_quantity: req.ballot_quantity,
        proof_id: req.proof_id,
        security_features: req.security_features,
        status: "ordered".to_string(),
        printer_reference: format!("PRN-{}", Uuid::now_v7()),
        created_at: now,
        updated_at: now,
    };
    state.orders.insert(order.order_id, order.clone());
    state.audit.lock().expect("lock poisoned").push(AuditEvent {
        event_id: Uuid::now_v7(),
        event_type: "order_created".to_string(),
        order_id: Some(order.order_id),
        actor_role: None,
        payload_summary: serde_json::json!({ "order_id": order.order_id }),
        created_at: now,
    });
    (StatusCode::CREATED, Json(order))
}

async fn list_print_orders(State(state): State<SharedState>) -> impl IntoResponse {
    let mut items = state.orders.values();
    items.sort_by(|a, b| a.order_id.cmp(&b.order_id));
    Json(json!({ "orders": items }))
}

async fn get_print_order(
    State(state): State<SharedState>,
    Path(order_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.orders.get(&order_id) {
        Some(o) => Json(o.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "order not found" }))).into_response(),
    }
}

async fn submit_qc_decision(
    State(state): State<SharedState>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<QCDecision>,
) -> impl IntoResponse {
    let res = state.orders.with_lock(|m| {
        match m.get_mut(&order_id) {
            Some(o) => {
                o.status = if req.passed { "qc".to_string() } else { "ordered".to_string() };
                o.updated_at = Utc::now();
                Some(o.clone())
            }
            None => None,
        }
    });
    match res {
        Some(o) => Json(o).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "order not found" }))).into_response(),
    }
}

async fn record_ballot_delivery(
    State(state): State<SharedState>,
    Json(req): Json<DeliveryRecordRequest>,
) -> impl IntoResponse {
    let now = Utc::now();
    let delivery = DeliveryRecord {
        delivery_id: Uuid::now_v7(),
        order_id: req.order_id,
        delivered_quantity: req.delivered_quantity,
        serial_range_start: req.serial_range_start,
        serial_range_end: req.serial_range_end,
        received_by_hash: req.received_by_hash,
        received_at: req.received_at,
        status: "received".to_string(),
        created_at: now,
    };
    state.deliveries.insert(delivery.delivery_id, delivery.clone());
    state.audit.lock().expect("lock poisoned").push(AuditEvent {
        event_id: Uuid::now_v7(),
        event_type: "delivery_recorded".to_string(),
        order_id: Some(delivery.order_id),
        actor_role: None,
        payload_summary: serde_json::to_value(&delivery).unwrap_or(serde_json::Value::Null),
        created_at: now,
    });
    (StatusCode::CREATED, Json(delivery))
}

async fn get_printing_audit_trail(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.audit.lock().expect("lock poisoned");
    let items = guard.clone();
    drop(guard);
    Json(json!({ "audit_trail": items }))
}
