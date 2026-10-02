// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::spec::{DeliveryRecordRequest, PrintOrderRequest, QCDecision};
use crate::store::Store;

/// Postgres-backed application state (no in-memory).
///
/// Production (`Store::Pg`) uses `PgPool` against `printing_api_*` tables
/// (`migrations/V053__printing_api_surface.sql`). Hermetic tests use
/// `Store::Mem` only.
#[derive(Clone)]
pub struct AppState {
    /// store.
    pub store: Store,
}

impl Default for AppState {
    fn default() -> Self {
        Self { store: Store::mem() }
    }
}

/// SharedState.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// new_shared (hermetic, in-memory).
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }

    /// new from postgres pool (production).
    pub fn from_pool(pool: PgPool) -> SharedState {
        Arc::new(Self { store: Store::pg(pool) })
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

fn store_err(e: crate::store::StoreError) -> axum::response::Response {
    match e {
        crate::store::StoreError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": "not found"})),
        )
            .into_response(),
        crate::store::StoreError::Db(err) => {
            tracing::error!(error = %err, "store db error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": "internal error"})),
            )
                .into_response()
        }
    }
}

async fn create_print_order(
    State(state): State<SharedState>,
    Json(req): Json<PrintOrderRequest>,
) -> impl IntoResponse {
    match state.store.create_order(req).await {
        Ok(order) => (StatusCode::CREATED, Json(order)).into_response(),
        Err(e) => store_err(e),
    }
}

async fn list_print_orders(State(state): State<SharedState>) -> impl IntoResponse {
    match state.store.list_orders().await {
        Ok(items) => Json(json!({ "orders": items })).into_response(),
        Err(e) => store_err(e),
    }
}

async fn get_print_order(
    State(state): State<SharedState>,
    Path(order_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.store.get_order(order_id).await {
        Ok(Some(o)) => Json(o).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "order not found" }))).into_response(),
        Err(e) => store_err(e),
    }
}

async fn submit_qc_decision(
    State(state): State<SharedState>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<QCDecision>,
) -> impl IntoResponse {
    match state.store.submit_qc(order_id, req).await {
        Ok(Some(o)) => Json(o).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "detail": "order not found" }))).into_response(),
        Err(e) => store_err(e),
    }
}

async fn record_ballot_delivery(
    State(state): State<SharedState>,
    Json(req): Json<DeliveryRecordRequest>,
) -> impl IntoResponse {
    match state.store.record_delivery(req).await {
        Ok(delivery) => (StatusCode::CREATED, Json(delivery)).into_response(),
        Err(e) => store_err(e),
    }
}

async fn get_printing_audit_trail(State(state): State<SharedState>) -> impl IntoResponse {
    match state.store.list_audit().await {
        Ok(items) => Json(json!({ "audit_trail": items })).into_response(),
        Err(e) => store_err(e),
    }
}
