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

//! Public contract handlers (inventory/management.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::store::Store;

/// Typed shortfall creation request (derived from inventory/management.yaml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortfallRequest {
    /// Constituency code reporting the shortfall.
    pub constituency_code: String,
    /// Item type (e.g. ballot_papers, seals, ink).
    pub item: String,
    /// Quantity missing.
    pub quantity: u32,
    /// Reporting officer hash.
    pub reported_by_hash: String,
}

/// Typed shortfall record stored in memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortfall {
    /// Unique shortfall id.
    pub shortfall_id: String,
    /// Status.
    pub status: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Original request.
    pub request: ShortfallRequest,
}

fn page(items: &Value) -> Value {
    eel_spec_helpers_rs::page_value(items)
}

/// GET /api/v1/inventory/kits — kit allocation summary.
pub async fn list_kit_allocation_summary() -> impl IntoResponse {
    Json(page(&json!([])))
}

/// GET /api/v1/inventory/kits/{constituency_code}/allocation — constituency kit allocation.
pub async fn get_constituency_kit_allocation(Path(code): Path<String>) -> impl IntoResponse {
    Json(json!({
        "constituency_code": code,
        "total_kits_allocated": 0,
        "kits_delivered": 0,
        "kits_pending": 0,
        "polling_stations": 0,
    }))
}

/// GET /api/v1/inventory/materials/{constituency_code}/status — material status.
pub async fn get_material_status(Path(code): Path<String>) -> impl IntoResponse {
    Json(json!({
        "constituency_code": code,
        "ballot_papers_allocated": 0,
        "ballot_papers_received": 0,
        "ink_units": 0,
        "seals": 0,
    }))
}

/// POST /api/v1/inventory/shortfalls — report a shortfall (postgres).
pub async fn report_shortfall(
    State(store): State<Store>,
    Json(req): Json<ShortfallRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let rec = Shortfall {
        shortfall_id: id.clone(),
        status: "reported".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        request: req,
    };
    match store.insert_shortfall(&rec).await {
        Ok(()) => (StatusCode::CREATED, Json(rec)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "shortfall insert failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": "internal error"})),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/inventory/shortfalls — list shortfalls (postgres).
pub async fn list_shortfalls(State(store): State<Store>) -> impl IntoResponse {
    match store.list_shortfalls().await {
        Ok(items) => Json(page(&serde_json::to_value(items).unwrap())).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "shortfall list failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": "internal error"})),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/inventory/audit-trail — inventory audit trail.
pub async fn get_inventory_audit_trail() -> impl IntoResponse {
    Json(page(&json!([])))
}
