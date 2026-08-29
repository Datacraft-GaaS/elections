//! Public contract handlers (custody/openapi.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::spec;
use crate::state::AppState;

fn page(items: &serde_json::Value) -> serde_json::Value {
    eel_spec_helpers_rs::page_total_cursor_value(items)
}

/// POST /api/v1/custody/assets — register an asset.
pub async fn register_asset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<spec::AssetCreate>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().to_rfc3339();
    let asset = spec::Asset {
        asset_id: uuid::Uuid::now_v7().to_string(),
        asset_type: req.asset_type,
        serial_number: req.serial_number,
        current_custodian_id: req.current_custodian_id,
        current_location: req.current_location,
        status: "in_custody".to_string(),
        condition: req.condition,
        created_at: now.clone(),
        updated_at: now,
    };
    state.assets.insert(asset.asset_id.clone(), asset.clone());
    (StatusCode::CREATED, Json(asset))
}

/// GET /api/v1/custody/assets — list assets.
pub async fn list_assets(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut items = state.assets.values();
    items.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));
    Json(page(&serde_json::to_value(items).unwrap()))
}

/// PATCH /api/v1/custody/assets/{id}/custody — transfer custody.
pub async fn transfer_custody(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<spec::CustodyTransferCreate>,
) -> impl IntoResponse {
    let (from_custodian, from_location) = {
        match state.assets.get(&id) {
            Some(a) => (a.current_custodian_id.clone(), a.current_location.clone()),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "detail": "asset not found" })),
                )
                    .into_response()
            }
        }
    };
    let transfer = spec::CustodyTransfer {
        transfer_id: uuid::Uuid::now_v7().to_string(),
        asset_id: id.clone(),
        from_custodian_id: from_custodian,
        to_custodian_id: req.to_custodian_id.clone(),
        from_location,
        to_location: req.to_location.clone(),
        reason: req.reason,
        transferred_at: chrono::Utc::now().to_rfc3339(),
        officer_id: req.officer_id,
        witness_id: req.witness_id,
    };
    state.assets.with_lock(|m| {
        if let Some(a) = m.get_mut(&id) {
            a.current_custodian_id = transfer.to_custodian_id.clone();
            a.current_location = transfer.to_location.clone();
            a.status = "in_transit".to_string();
            a.updated_at = chrono::Utc::now().to_rfc3339();
        }
    });
    state
        .transfers
        .with_lock(|m| m.entry(id).or_default().push(transfer.clone()));
    (StatusCode::CREATED, Json(transfer)).into_response()
}

/// GET /api/v1/custody/assets/{id}/history — asset custody history.
pub async fn get_asset_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let items = state.transfers.get(&id).unwrap_or_default();
    Json(page(&serde_json::to_value(items).unwrap()))
}

/// POST /api/v1/custody/seals — create a seal.
pub async fn create_seal(
    State(state): State<Arc<AppState>>,
    Json(req): Json<spec::SealCreate>,
) -> impl IntoResponse {
    let seal = spec::Seal {
        seal_id: uuid::Uuid::now_v7().to_string(),
        asset_id: req.asset_id,
        seal_number: req.seal_number,
        sealed_by_id: req.sealed_by_id,
        sealed_at: chrono::Utc::now().to_rfc3339(),
        status: "intact".to_string(),
    };
    state.seals.insert(seal.seal_id.clone(), seal.clone());
    (StatusCode::CREATED, Json(seal))
}

/// PATCH /api/v1/custody/seals/{id}/break — break a seal.
pub async fn break_seal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<spec::SealBreakCreate>,
) -> impl IntoResponse {
    {
        let ok = state.seals.with_lock(|m| match m.get_mut(&id) {
            Some(s) => {
                s.status = "broken".to_string();
                true
            }
            None => false,
        });
        if !ok {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "detail": "seal not found" })),
            )
                .into_response();
        }
    }
    let brk = spec::SealBreak {
        break_id: uuid::Uuid::now_v7().to_string(),
        seal_id: id.clone(),
        broken_by_id: req.broken_by_id,
        countersign_id: req.countersign_id,
        reason: req.reason,
        broken_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .seal_breaks
        .with_lock(|m| m.entry(id).or_default().push(brk.clone()));
    (StatusCode::CREATED, Json(brk)).into_response()
}

/// GET /api/v1/custody/reserves — list reserves.
pub async fn list_reserves(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut items = state.reserves.values();
    items.sort_by(|a, b| a.reserve_id.cmp(&b.reserve_id));
    Json(page(&serde_json::to_value(items).unwrap()))
}

/// GET /api/v1/custody/summary — custody summary.
pub async fn get_custody_summary(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (total, in_custody, in_transit) = state.assets.with_lock(|m| {
        let total = m.len() as u32;
        let in_custody = m.values().filter(|a| a.status == "in_custody").count() as u32;
        let in_transit = m.values().filter(|a| a.status == "in_transit").count() as u32;
        (total, in_custody, in_transit)
    });
    Json(spec::CustodySummary {
        total_assets: total,
        assets_in_custody: in_custody,
        assets_in_transit: in_transit,
        broken_seals_today: 0,
        discrepancy_count: 0,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// POST /api/v1/custody/reconcile — start reconciliation.
pub async fn start_reconciliation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<spec::ReconcileCreate>,
) -> impl IntoResponse {
    let status = if req.discrepancies.is_empty() {
        "reconciled"
    } else {
        "discrepancy_found"
    };
    let report = spec::ReconcileReport {
        reconcile_id: uuid::Uuid::now_v7().to_string(),
        station_code: req.station_code,
        election_id: req.election_id,
        assets_expected: req.assets_expected,
        assets_found: req.assets_found,
        discrepancies: req.discrepancies,
        status: status.to_string(),
        officer_id: req.officer_id,
    };
    state.reconciles.insert(report.reconcile_id.clone(), report.clone());
    (StatusCode::CREATED, Json(report))
}

/// GET /api/v1/custody/reconcile/{id}/report — reconciliation report.
pub async fn get_reconcile_report(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.reconciles.get(&id) {
        Some(r) => Json(r.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "reconciliation not found" })),
        )
            .into_response(),
    }
}
