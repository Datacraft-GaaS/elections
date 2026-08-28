//! Custody endpoints (M20): asset register and custody-chain events.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::handlers::{db_error, problem};
use crate::store::Store;
use custody::domain::{Asset, AssetStatus, AssetType, CustodyEvent};

fn parse_asset_type(s: &str) -> Option<AssetType> {
    match s {
        "kit" => Some(AssetType::Kit),
        "ballot" => Some(AssetType::Ballot),
        "seal" => Some(AssetType::Seal),
        "consumable" => Some(AssetType::Consumable),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub asset_type: String,
    pub serial: String,
    pub model: String,
}

pub async fn create_asset(
    State(store): State<Store>,
    Json(req): Json<CreateAssetRequest>,
) -> Response {
    let Some(asset_type) = parse_asset_type(&req.asset_type) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "asset_type must be kit|ballot|seal|consumable",
        );
    };
    let asset = Asset {
        asset_id: Uuid::now_v7().to_string(),
        asset_type,
        serial: req.serial,
        model: req.model,
        status: AssetStatus::New,
        station_code: None,
        last_seen_at: Utc::now(),
    };
    if let Err(e) = store.insert_asset(&asset).await {
        return db_error(e);
    }
    (StatusCode::CREATED, Json(asset)).into_response()
}

pub async fn list_assets(State(store): State<Store>) -> Response {
    match store.list_assets().await {
        Ok(assets) => Json(serde_json::json!({ "assets": assets })).into_response(),
        Err(e) => db_error(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCustodyEventRequest {
    pub asset_id: String,
    pub from_custodian: String,
    pub to_custodian: String,
    pub location: String,
    pub signature: String,
}

pub async fn create_custody_event(
    State(store): State<Store>,
    Json(req): Json<CreateCustodyEventRequest>,
) -> Response {
    match store.get_asset(&req.asset_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return problem(StatusCode::NOT_FOUND, "asset not found"),
        Err(e) => return db_error(e),
    }
    let history = match store.custody_history(&req.asset_id).await {
        Ok(h) => h,
        Err(e) => return db_error(e),
    };
    let event = CustodyEvent {
        event_id: Uuid::now_v7().to_string(),
        asset_id: req.asset_id,
        from_custodian: req.from_custodian,
        to_custodian: req.to_custodian,
        location: req.location,
        ts_hlc: Utc::now().timestamp_millis(),
        signature: req.signature,
        seq: history.len() as u64 + 1,
    };
    if let Err(e) = store.insert_custody_event(&event).await {
        return db_error(e);
    }
    (StatusCode::CREATED, Json(event)).into_response()
}

pub async fn asset_history(State(store): State<Store>, Path(id): Path<String>) -> Response {
    match store.custody_history(&id).await {
        Ok(events) => Json(serde_json::json!({ "events": events })).into_response(),
        Err(e) => db_error(e),
    }
}
