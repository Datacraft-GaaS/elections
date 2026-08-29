//! Public contract handlers (custody/openapi.yaml) — postgres-backed (no in-memory).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::spec;
use crate::state::AppState;
use crate::store::StoreError;

fn page(items: &serde_json::Value) -> serde_json::Value {
    eel_spec_helpers_rs::page_total_cursor_value(items)
}

fn store_err(e: StoreError) -> axum::response::Response {
    match e {
        StoreError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "not found"})),
        )
            .into_response(),
        StoreError::Db(err) => {
            tracing::error!(error = %err, "store db error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "detail": "internal error"})),
            )
                .into_response()
        }
    }
}

/// POST /api/v1/custody/assets — register an asset.
pub async fn register_asset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<spec::AssetCreate>,
) -> impl IntoResponse {
    match state.store.create_asset(req).await {
        Ok(asset) => (StatusCode::CREATED, Json(asset)).into_response(),
        Err(e) => store_err(e),
    }
}

/// GET /api/v1/custody/assets — list assets.
pub async fn list_assets(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.store.list_assets().await {
        Ok(mut items) => {
            items.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));
            Json(page(&serde_json::to_value(items).unwrap())).into_response()
        }
        Err(e) => store_err(e),
    }
}

/// PATCH /api/v1/custody/assets/{id}/custody — transfer custody.
pub async fn transfer_custody(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<spec::CustodyTransferCreate>,
) -> impl IntoResponse {
    match state.store.transfer(&id, req).await {
        Ok(t) => (StatusCode::CREATED, Json(t)).into_response(),
        Err(e) => store_err(e),
    }
}

/// GET /api/v1/custody/assets/{id}/history — asset custody history.
pub async fn get_asset_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.store.asset_history(&id).await {
        Ok(items) => Json(page(&serde_json::to_value(items).unwrap())).into_response(),
        Err(e) => store_err(e),
    }
}

/// POST /api/v1/custody/seals — create a seal.
pub async fn create_seal(
    State(state): State<Arc<AppState>>,
    Json(req): Json<spec::SealCreate>,
) -> impl IntoResponse {
    match state.store.create_seal(req).await {
        Ok(seal) => (StatusCode::CREATED, Json(seal)).into_response(),
        Err(e) => store_err(e),
    }
}

/// PATCH /api/v1/custody/seals/{id}/break — break a seal.
pub async fn break_seal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<spec::SealBreakCreate>,
) -> impl IntoResponse {
    match state.store.break_seal(&id, req).await {
        Ok(brk) => (StatusCode::CREATED, Json(brk)).into_response(),
        Err(e) => store_err(e),
    }
}

/// GET /api/v1/custody/reserves — list reserves.
pub async fn list_reserves(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.store.list_reserves().await {
        Ok(mut items) => {
            items.sort_by(|a, b| a.reserve_id.cmp(&b.reserve_id));
            Json(page(&serde_json::to_value(items).unwrap())).into_response()
        }
        Err(e) => store_err(e),
    }
}

/// GET /api/v1/custody/summary — custody summary.
pub async fn get_custody_summary(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.store.custody_summary().await {
        Ok(s) => Json(s).into_response(),
        Err(e) => store_err(e),
    }
}

/// POST /api/v1/custody/reconcile — start reconciliation.
pub async fn start_reconciliation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<spec::ReconcileCreate>,
) -> impl IntoResponse {
    match state.store.create_reconcile(req).await {
        Ok(report) => (StatusCode::CREATED, Json(report)).into_response(),
        Err(e) => store_err(e),
    }
}

/// GET /api/v1/custody/reconcile/{id}/report — reconciliation report.
pub async fn get_reconcile_report(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.store.get_reconcile(&id).await {
        Ok(Some(r)) => Json(r).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "reconciliation not found" })),
        )
            .into_response(),
        Err(e) => store_err(e),
    }
}
