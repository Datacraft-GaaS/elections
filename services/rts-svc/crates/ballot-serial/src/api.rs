//! Axum sub-router for ballot-serial (WP-3.3, M63).
//! Merged into rts-api at boot via Router::merge().
//! Path prefix: /api/v1/ballot-serial

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ---- request/response models ------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AssignSerialRequest {
    pub lot_id:        String,
    pub station_code:  String,
    pub serial_low:    u64,
    pub serial_high:   u64,
    pub assigned_by:   String,
    pub manifest_ref:  String,
    pub manifest_sig:  String,
}

#[derive(Debug, Serialize)]
pub struct AssignSerialResponse {
    pub assignment_id: String,
    pub station_code:  String,
    pub serial_low:    u64,
    pub serial_high:   u64,
    pub status:        String,
}

#[derive(Debug, Deserialize)]
pub struct ReceiptRequest {
    pub assignment_id: String,
    pub clerk_id:      String,
    pub receipt_hash:  String,
}

#[derive(Debug, Deserialize)]
pub struct SpoiltRequest {
    pub station_code:       String,
    pub spoilt_serial:      u64,
    pub replacement_serial: Option<u64>,
    pub reason:             String,
    pub clerk_id:           String,
}

#[derive(Debug, Deserialize)]
pub struct ReconcileRequest {
    pub station_code:    String,
    pub election_id:     String,
    pub deposited_count: u64,
    pub spoilt_count:    u64,
}

#[derive(Debug, Serialize)]
pub struct ReconcileResponse {
    pub reconciliation_id: String,
    pub station_code:      String,
    pub status:            String,
    pub result_hold:       bool,
    pub flag_reason:       Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveHoldRequest {
    pub resolved_by_po:  String,
    pub resolved_by_cro: String,
    pub reason:          String,
}

// ---- handlers ---------------------------------------------------------------

async fn assign_serial(Json(req): Json<AssignSerialRequest>) -> impl IntoResponse {
    let id = Uuid::now_v7().to_string();
    info!(
        station_code = %req.station_code,
        lot_id = %req.lot_id,
        serial_low = req.serial_low,
        serial_high = req.serial_high,
        "ballot-serial: assign"
    );
    (
        StatusCode::CREATED,
        Json(AssignSerialResponse {
            assignment_id: id,
            station_code: req.station_code,
            serial_low: req.serial_low,
            serial_high: req.serial_high,
            status: "assigned".to_string(),
        }),
    )
}

async fn confirm_receipt(Json(req): Json<ReceiptRequest>) -> impl IntoResponse {
    info!(assignment_id = %req.assignment_id, clerk_id = %req.clerk_id, "ballot-serial: receipt");
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "receipt_id": Uuid::now_v7().to_string(),
            "assignment_id": req.assignment_id,
            "status": "issued",
        })),
    )
}

async fn record_spoilt(Json(req): Json<SpoiltRequest>) -> impl IntoResponse {
    info!(
        station_code = %req.station_code,
        spoilt_serial = req.spoilt_serial,
        "ballot-serial: spoilt"
    );
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "station_code": req.station_code,
            "spoilt_serial": req.spoilt_serial,
            "status": "recorded",
        })),
    )
}

async fn trigger_reconciliation(Json(req): Json<ReconcileRequest>) -> impl IntoResponse {
    use crate::reconciliation::{StationCounts, reconcile};

    // We don't have the serial range in the request (needs a DB lookup in full impl).
    // For the stub, return a synthetic result and flag if totals look impossible.
    let counts = StationCounts {
        station_code: req.station_code.clone(),
        serial_low: 1,
        serial_high: req.deposited_count + req.spoilt_count,
        deposited_count: req.deposited_count,
        spoilt_count: req.spoilt_count,
    };
    use crate::reconciliation::ReconciliationStatus;
    let result = reconcile(&counts);
    let rec_id = Uuid::now_v7().to_string();
    let is_held = matches!(result.status, ReconciliationStatus::Held { .. });

    info!(
        station_code = %req.station_code,
        result_hold = is_held,
        "ballot-serial: reconcile"
    );

    (
        StatusCode::ACCEPTED,
        Json(ReconcileResponse {
            reconciliation_id: rec_id,
            station_code: req.station_code,
            status: if is_held { "held" } else { "cleared" }.to_string(),
            result_hold: is_held,
            flag_reason: result.flag_reason,
        }),
    )
}

async fn get_reconciliation(Path(station_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "station_code": station_id,
        "status": "pending",
        "result_hold": false,
    }))
}

async fn resolve_hold(
    Path(id): Path<String>,
    Json(req): Json<ResolveHoldRequest>,
) -> impl IntoResponse {
    info!(
        reconciliation_id = %id,
        resolved_by_po = %req.resolved_by_po,
        resolved_by_cro = %req.resolved_by_cro,
        "ballot-serial: resolve hold"
    );
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "reconciliation_id": id,
            "status": "cleared",
            "resolved_at": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

async fn list_assignments() -> impl IntoResponse {
    Json(serde_json::json!({ "assignments": [] }))
}

// ---- router -----------------------------------------------------------------

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/ballot-serial/assignments", post(assign_serial))
        .route("/api/v1/ballot-serial/assignments", get(list_assignments))
        .route("/api/v1/ballot-serial/receipts", post(confirm_receipt))
        .route("/api/v1/ballot-serial/spoilt", post(record_spoilt))
        .route("/api/v1/ballot-serial/reconcile", post(trigger_reconciliation))
        .route(
            "/api/v1/ballot-serial/reconcile/:station_id",
            get(get_reconciliation),
        )
        .route("/api/v1/ballot-serial/reconcile/:id/resolve", put(resolve_hold))
}
