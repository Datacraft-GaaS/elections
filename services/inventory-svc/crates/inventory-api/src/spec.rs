//! Public contract handlers (inventory/management.yaml).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

fn shortfalls() -> &'static Mutex<HashMap<String, Shortfall>> {
    static STORE: OnceLock<Mutex<HashMap<String, Shortfall>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
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

/// POST /api/v1/inventory/shortfalls — report a shortfall.
pub async fn report_shortfall(Json(req): Json<ShortfallRequest>) -> impl IntoResponse {
    let id = uuid::Uuid::now_v7().to_string();
    let rec = Shortfall {
        shortfall_id: id.clone(),
        status: "reported".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        request: req,
    };
    shortfalls().lock().expect("lock poisoned").insert(id, rec.clone());
    (StatusCode::CREATED, Json(rec))
}

/// GET /api/v1/inventory/shortfalls — list shortfalls.
pub async fn list_shortfalls() -> impl IntoResponse {
    let guard = shortfalls().lock().expect("lock poisoned");
    let items: Vec<Shortfall> = guard.values().cloned().collect();
    drop(guard);
    Json(page(&serde_json::to_value(items).unwrap()))
}

/// GET /api/v1/inventory/audit-trail — inventory audit trail.
pub async fn get_inventory_audit_trail() -> impl IntoResponse {
    Json(page(&json!([])))
}
