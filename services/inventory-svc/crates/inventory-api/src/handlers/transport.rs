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

//! Transport & security-deployment endpoints (M20).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::handlers::{db_error, problem, transport_error};
use crate::store::Store;
use transport::domain::{
    assign_security, AssignmentTarget, LoadItem, ScanMethod, Vehicle, VehicleLoad, VehicleStatus,
    VehicleType,
};

fn parse_vehicle_type(s: &str) -> Option<VehicleType> {
    match s {
        "truck" => Some(VehicleType::Truck),
        "van" => Some(VehicleType::Van),
        "air" => Some(VehicleType::Air),
        _ => None,
    }
}

fn parse_scan_method(s: &str) -> Option<ScanMethod> {
    match s {
        "rfid" => Some(ScanMethod::Rfid),
        "barcode" => Some(ScanMethod::Barcode),
        _ => None,
    }
}

fn parse_target(s: &str) -> Option<AssignmentTarget> {
    match s {
        "vehicle" => Some(AssignmentTarget::Vehicle),
        "station" => Some(AssignmentTarget::Station),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Vehicles
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateVehicleRequest {
    pub registration: String,
    pub vehicle_type: String,
    pub capacity: u64,
}

pub async fn create_vehicle(
    State(store): State<Store>,
    Json(req): Json<CreateVehicleRequest>,
) -> Response {
    let Some(vehicle_type) = parse_vehicle_type(&req.vehicle_type) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "vehicle_type must be truck|van|air",
        );
    };
    if req.capacity == 0 {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "capacity must be positive",
        );
    }
    let vehicle = Vehicle {
        id: Uuid::now_v7().to_string(),
        registration: req.registration,
        vehicle_type,
        capacity: req.capacity,
        status: VehicleStatus::Available,
    };
    if let Err(e) = store.insert_vehicle(&vehicle).await {
        return db_error(e);
    }
    (StatusCode::CREATED, Json(vehicle)).into_response()
}

pub async fn list_vehicles(State(store): State<Store>) -> Response {
    match store.list_vehicles().await {
        Ok(vehicles) => Json(serde_json::json!({ "vehicles": vehicles })).into_response(),
        Err(e) => db_error(e),
    }
}

// ---------------------------------------------------------------------------
// Loads
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateLoadRequest {
    pub vehicle_id: String,
    pub destination: String,
    #[serde(default)]
    pub escort_agent_id: Option<String>,
}

pub async fn create_load(
    State(store): State<Store>,
    Json(req): Json<CreateLoadRequest>,
) -> Response {
    match store.get_vehicle(&req.vehicle_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return problem(StatusCode::NOT_FOUND, "vehicle not found"),
        Err(e) => return db_error(e),
    }
    let load = VehicleLoad::new(
        Uuid::now_v7().to_string(),
        req.vehicle_id,
        req.destination,
        req.escort_agent_id,
        Utc::now(),
    );
    if let Err(e) = store.insert_load(&load).await {
        return db_error(e);
    }
    (StatusCode::CREATED, Json(load)).into_response()
}

#[derive(Debug, Deserialize)]
pub struct AddItemRequest {
    pub lot_id: String,
    pub quantity: u64,
    pub scan_method: String,
    pub scan_value: String,
    pub scanned_by: String,
}

pub async fn add_load_item(
    State(store): State<Store>,
    Path(id): Path<String>,
    Json(req): Json<AddItemRequest>,
) -> Response {
    let mut load = match store.get_load(&id).await {
        Ok(Some(l)) => l,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "load not found"),
        Err(e) => return db_error(e),
    };
    let vehicle = match store.get_vehicle(&load.vehicle_id).await {
        Ok(Some(v)) => v,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "vehicle not found"),
        Err(e) => return db_error(e),
    };
    let Some(scan_method) = parse_scan_method(&req.scan_method) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "scan_method must be rfid|barcode",
        );
    };
    let item = LoadItem {
        id: Uuid::now_v7().to_string(),
        lot_id: req.lot_id,
        quantity: req.quantity,
        scan_method,
        scan_value: req.scan_value,
        scanned_by: req.scanned_by,
        scanned_at: Utc::now(),
    };
    if let Err(e) = load.add_item(&vehicle, item) {
        return transport_error(&e);
    }
    if let Err(e) = store.insert_load(&load).await {
        return db_error(e);
    }
    Json(load).into_response()
}

pub async fn dispatch_load(State(store): State<Store>, Path(id): Path<String>) -> Response {
    let mut load = match store.get_load(&id).await {
        Ok(Some(l)) => l,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "load not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = load.dispatch(Utc::now()) {
        return transport_error(&e);
    }
    if let Err(e) = store.insert_load(&load).await {
        return db_error(e);
    }
    Json(load).into_response()
}

pub async fn deliver_load(State(store): State<Store>, Path(id): Path<String>) -> Response {
    let mut load = match store.get_load(&id).await {
        Ok(Some(l)) => l,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "load not found"),
        Err(e) => return db_error(e),
    };
    if let Err(e) = load.deliver(Utc::now()) {
        return transport_error(&e);
    }
    if let Err(e) = store.insert_load(&load).await {
        return db_error(e);
    }
    Json(load).into_response()
}

// ---------------------------------------------------------------------------
// Security assignments
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateSecurityAssignmentRequest {
    pub agent_id: String,
    pub target: String,
    pub target_id: String,
    pub starts_at: chrono::DateTime<Utc>,
    pub ends_at: chrono::DateTime<Utc>,
}

pub async fn create_security_assignment(
    State(store): State<Store>,
    Json(req): Json<CreateSecurityAssignmentRequest>,
) -> Response {
    let Some(target) = parse_target(&req.target) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "target must be vehicle|station",
        );
    };
    let existing = match store.list_assignments().await {
        Ok(a) => a,
        Err(e) => return db_error(e),
    };
    match assign_security(
        Uuid::now_v7().to_string(),
        req.agent_id,
        target,
        req.target_id,
        req.starts_at,
        req.ends_at,
        &existing,
    ) {
        Ok(assignment) => {
            if let Err(e) = store.insert_assignment(&assignment).await {
                return db_error(e);
            }
            (StatusCode::CREATED, Json(assignment)).into_response()
        }
        Err(e) => transport_error(&e),
    }
}

pub async fn list_security_assignments(State(store): State<Store>) -> Response {
    match store.list_assignments().await {
        Ok(assignments) => Json(serde_json::json!({ "assignments": assignments })).into_response(),
        Err(e) => db_error(e),
    }
}
