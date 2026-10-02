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

//! Warehouse, transfer, pallet, and incident endpoints (M20).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::handlers::{db_error, problem, warehouse_error};
use crate::store::Store;
use warehouse::domain::{
    Incident, IncidentSeverity, IncidentType, ItemType, Pallet, PalletItem, Warehouse,
    WarehouseTier, WarehouseTransfer,
};

fn parse_tier(s: &str) -> Option<WarehouseTier> {
    match s {
        "master" => Some(WarehouseTier::Master),
        "regional" => Some(WarehouseTier::Regional),
        "county" => Some(WarehouseTier::County),
        _ => None,
    }
}

fn parse_item_type(s: &str) -> Option<ItemType> {
    match s {
        "ballot_box" => Some(ItemType::BallotBox),
        "ballot_lot" => Some(ItemType::BallotLot),
        "kit" => Some(ItemType::Kit),
        "consumable" => Some(ItemType::Consumable),
        _ => None,
    }
}

fn parse_incident_type(s: &str) -> Option<IncidentType> {
    match s {
        "tamper" => Some(IncidentType::Tamper),
        "loss" => Some(IncidentType::Loss),
        "damage" => Some(IncidentType::Damage),
        "discrepancy" => Some(IncidentType::Discrepancy),
        "other" => Some(IncidentType::Other),
        _ => None,
    }
}

fn parse_severity(s: &str) -> Option<IncidentSeverity> {
    match s {
        "P0" => Some(IncidentSeverity::P0),
        "P1" => Some(IncidentSeverity::P1),
        "P2" => Some(IncidentSeverity::P2),
        "P3" => Some(IncidentSeverity::P3),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Warehouses
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateWarehouseRequest {
    pub code: String,
    pub name: String,
    pub tier: String,
    pub latitude: f64,
    pub longitude: f64,
    pub geofence_radius_m: f64,
}

pub async fn create_warehouse(
    State(store): State<Store>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Response {
    let Some(tier) = parse_tier(&req.tier) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "tier must be master|regional|county",
        );
    };
    match Warehouse::new(
        Uuid::now_v7().to_string(),
        req.code,
        req.name,
        tier,
        req.latitude,
        req.longitude,
        req.geofence_radius_m,
    ) {
        Ok(w) => {
            if let Err(e) = store.insert_warehouse(&w).await {
                return db_error(e);
            }
            (StatusCode::CREATED, Json(w)).into_response()
        }
        Err(e) => warehouse_error(&e),
    }
}

pub async fn list_warehouses(State(store): State<Store>) -> Response {
    match store.list_warehouses().await {
        Ok(warehouses) => Json(serde_json::json!({ "warehouses": warehouses })).into_response(),
        Err(e) => db_error(e),
    }
}

// ---------------------------------------------------------------------------
// Transfers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateTransferRequest {
    pub from_warehouse_id: String,
    pub to_warehouse_id: String,
    #[serde(default)]
    pub vehicle_id: Option<String>,
    #[serde(default)]
    pub expected_pallets: Vec<String>,
}

pub async fn create_transfer(
    State(store): State<Store>,
    Json(req): Json<CreateTransferRequest>,
) -> Response {
    match WarehouseTransfer::new(
        Uuid::now_v7().to_string(),
        req.from_warehouse_id,
        req.to_warehouse_id,
        req.vehicle_id,
        req.expected_pallets,
        Utc::now(),
    ) {
        Ok(t) => {
            if let Err(e) = store.insert_transfer(&t).await {
                return db_error(e);
            }
            (StatusCode::CREATED, Json(t)).into_response()
        }
        Err(e) => warehouse_error(&e),
    }
}

pub async fn depart_transfer(State(store): State<Store>, Path(id): Path<String>) -> Response {
    let mut t = match store.get_transfer(&id).await {
        Ok(Some(t)) => t,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "transfer not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = t.depart(Utc::now()) {
        return warehouse_error(&e);
    }
    if let Err(e) = store.insert_transfer(&t).await {
        return db_error(e);
    }
    Json(t).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ArriveTransferRequest {
    #[serde(default)]
    pub arrived_pallets: Vec<String>,
}

pub async fn arrive_transfer(
    State(store): State<Store>,
    Path(id): Path<String>,
    Json(req): Json<ArriveTransferRequest>,
) -> Response {
    let mut t = match store.get_transfer(&id).await {
        Ok(Some(t)) => t,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "transfer not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = t.arrive(req.arrived_pallets, Utc::now()) {
        return warehouse_error(&e);
    }
    let missing = t.missing_pallets();
    let extra = t.extra_pallets();
    if let Err(e) = store.insert_transfer(&t).await {
        return db_error(e);
    }
    Json(serde_json::json!({
        "transfer": t,
        "reconciliation": { "missing_pallets": missing, "extra_pallets": extra },
    }))
    .into_response()
}

pub async fn get_transfer(State(store): State<Store>, Path(id): Path<String>) -> Response {
    let t = match store.get_transfer(&id).await {
        Ok(Some(t)) => t,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "transfer not found"),
        Err(e) => return db_error(e),
    };
    let missing = t.missing_pallets();
    let extra = t.extra_pallets();
    Json(serde_json::json!({
        "transfer": t,
        "reconciliation": { "missing_pallets": missing, "extra_pallets": extra },
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Pallets
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ReceivePalletRequest {
    pub serial: String,
    pub warehouse_id: String,
}

pub async fn receive_pallet(
    State(store): State<Store>,
    Json(req): Json<ReceivePalletRequest>,
) -> Response {
    let p = Pallet::receive(
        Uuid::now_v7().to_string(),
        req.serial,
        req.warehouse_id,
        Utc::now(),
    );
    if let Err(e) = store.insert_pallet(&p).await {
        return db_error(e);
    }
    (StatusCode::CREATED, Json(p)).into_response()
}

pub async fn store_pallet(State(store): State<Store>, Path(id): Path<String>) -> Response {
    let mut p = match store.get_pallet(&id).await {
        Ok(Some(p)) => p,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "pallet not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = p.store() {
        return warehouse_error(&e);
    }
    if let Err(e) = store.insert_pallet(&p).await {
        return db_error(e);
    }
    Json(p).into_response()
}

#[derive(Debug, Deserialize)]
pub struct BreakBulkItem {
    pub item_type: String,
    pub reference_id: String,
    pub quantity: u64,
}

#[derive(Debug, Deserialize)]
pub struct BreakBulkRequest {
    pub items: Vec<BreakBulkItem>,
}

pub async fn break_bulk_pallet(
    State(store): State<Store>,
    Path(id): Path<String>,
    Json(req): Json<BreakBulkRequest>,
) -> Response {
    let mut p = match store.get_pallet(&id).await {
        Ok(Some(p)) => p,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "pallet not found"),
        Err(e) => return db_error(e),
    };
    let mut items = Vec::with_capacity(req.items.len());
    for it in &req.items {
        let Some(item_type) = parse_item_type(&it.item_type) else {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "item_type must be ballot_box|ballot_lot|kit|consumable",
            );
        };
        items.push(PalletItem {
            id: Uuid::now_v7().to_string(),
            item_type,
            reference_id: it.reference_id.clone(),
            quantity: it.quantity,
        });
    }
    if let Err(e) = p.break_bulk(items) {
        return warehouse_error(&e);
    }
    if let Err(e) = store.insert_pallet(&p).await {
        return db_error(e);
    }
    Json(p).into_response()
}

pub async fn dispatch_pallet(State(store): State<Store>, Path(id): Path<String>) -> Response {
    let mut p = match store.get_pallet(&id).await {
        Ok(Some(p)) => p,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "pallet not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = p.dispatch() {
        return warehouse_error(&e);
    }
    if let Err(e) = store.insert_pallet(&p).await {
        return db_error(e);
    }
    Json(p).into_response()
}

#[derive(Debug, Deserialize)]
pub struct MarkUsedRequest {
    pub station_code: String,
}

pub async fn mark_pallet_used(
    State(store): State<Store>,
    Path(id): Path<String>,
    Json(req): Json<MarkUsedRequest>,
) -> Response {
    let mut p = match store.get_pallet(&id).await {
        Ok(Some(p)) => p,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "pallet not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = p.mark_used(req.station_code) {
        return warehouse_error(&e);
    }
    if let Err(e) = store.insert_pallet(&p).await {
        return db_error(e);
    }
    Json(p).into_response()
}

pub async fn get_pallet(State(store): State<Store>, Path(id): Path<String>) -> Response {
    match store.get_pallet(&id).await {
        Ok(Some(p)) => Json(p).into_response(),
        Ok(None) => problem(StatusCode::NOT_FOUND, "pallet not found"),
        Err(e) => db_error(e),
    }
}

// ---------------------------------------------------------------------------
// Incidents
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateIncidentRequest {
    #[serde(default)]
    pub warehouse_id: Option<String>,
    #[serde(default)]
    pub pallet_id: Option<String>,
    #[serde(default)]
    pub transfer_id: Option<String>,
    pub incident_type: String,
    pub severity: String,
    pub summary: String,
    pub reported_by: String,
}

pub async fn create_incident(
    State(store): State<Store>,
    Json(req): Json<CreateIncidentRequest>,
) -> Response {
    let Some(incident_type) = parse_incident_type(&req.incident_type) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "incident_type must be tamper|loss|damage|discrepancy|other",
        );
    };
    let Some(severity) = parse_severity(&req.severity) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "severity must be P0|P1|P2|P3",
        );
    };
    match Incident::new(
        Uuid::now_v7().to_string(),
        req.warehouse_id,
        req.pallet_id,
        req.transfer_id,
        incident_type,
        severity,
        req.summary,
        req.reported_by,
        Utc::now(),
    ) {
        Ok(i) => {
            if let Err(e) = store.insert_incident(&i).await {
                return db_error(e);
            }
            (StatusCode::CREATED, Json(i)).into_response()
        }
        Err(e) => warehouse_error(&e),
    }
}

pub async fn resolve_incident(State(store): State<Store>, Path(id): Path<String>) -> Response {
    let mut i = match store.get_incident(&id).await {
        Ok(Some(i)) => i,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "incident not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = i.resolve(Utc::now()) {
        return warehouse_error(&e);
    }
    if let Err(e) = store.insert_incident(&i).await {
        return db_error(e);
    }
    Json(i).into_response()
}

pub async fn list_incidents(State(store): State<Store>) -> Response {
    match store.list_incidents().await {
        Ok(incidents) => Json(serde_json::json!({ "incidents": incidents })).into_response(),
        Err(e) => db_error(e),
    }
}
