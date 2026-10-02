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

//! Axum sub-router for ballot-serial (WP-3.3, M63).
//! Merged into rts-api at boot via Router::merge().
//! Path prefix: /api/v1/ballot-serial
//!
//! Every handler reads or advances the real `BallotSerial` aggregate through
//! [`crate::store::AppState`].
//!
//! The serious one: `trigger_reconciliation` used to build its own
//! `StationCounts` with `serial_low: 1` and `serial_high: deposited + spoilt`, so
//! `assigned == accounted` by construction and
//! [`crate::reconciliation::reconcile`] could never report a mismatch. The hold
//! branch (FR-M63-024 / EC-DAT-11) was unreachable through the API even though the
//! function has tests proving it fires. A station could deposit more ballots than
//! it had been issued and be cleared. The range is now the one the station was
//! actually assigned.
//!
//! `get_reconciliation` returned `status: "pending", result_hold: false` for any
//! station code, so a station that had never been reconciled reported no hold —
//! which is the opposite of what a hold is for. `resolve_hold` returned
//! `cleared` for any id, clearing a hold that might not exist and with only one
//! of the two officers the request asks for.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::aggregate::BallotSerialStatus;
use crate::events::*;
use crate::reconciliation::{reconcile, ReconciliationStatus, StationCounts};
use crate::store::AppState;
use std::sync::Arc;

/// The state handle the router is built over.
type SharedState = Arc<AppState>;

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
    /// How many ballots the range covers, inclusive of both bounds.
    pub assigned_count: u64,
    /// Why the manifest signature is not verified here.
    pub blocking_reason: Option<String>,
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
    /// The arithmetic the result came from, so a reader can check it.
    pub assigned_count:    u64,
    pub deposited_count:   u64,
    pub spoilt_count:      u64,
    pub unused_count:      u64,
    pub discrepancy:       i64,
}

#[derive(Debug, Deserialize)]
pub struct ResolveHoldRequest {
    pub resolved_by_po:  String,
    pub resolved_by_cro: String,
    pub reason:          String,
}

// ---- helpers ----------------------------------------------------------------

/// A problem+json response.
fn problem(status: StatusCode, detail: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
}

// ---- handlers ---------------------------------------------------------------

async fn assign_serial(
    State(state): State<SharedState>,
    Json(req): Json<AssignSerialRequest>,
) -> axum::response::Response {
    info!(
        station_code = %req.station_code,
        lot_id = %req.lot_id,
        serial_low = req.serial_low,
        serial_high = req.serial_high,
        "ballot-serial: assign"
    );
    if req.station_code.trim().is_empty() || req.lot_id.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "station_code and lot_id are required",
        );
    }
    // An inverted range would report zero assigned ballots and silently clear
    // every reconciliation, so it is rejected rather than stored.
    if req.serial_high < req.serial_low {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!(
                "serial_high ({}) is below serial_low ({})",
                req.serial_high, req.serial_low
            ),
        );
    }
    // A station is issued one lot. Re-assigning would orphan the ballots already
    // issued against the previous range.
    if let Some(existing) = state.get(&req.station_code) {
        if existing.serial.status != BallotSerialStatus::Unassigned {
            return problem(
                StatusCode::CONFLICT,
                format!(
                    "station {} already has a ballot-serial assignment in range {}..={}",
                    req.station_code, existing.serial.serial_low, existing.serial.serial_high
                ),
            );
        }
    }
    let stream_id = {
        state.register(&req.station_code);
        state.get(&req.station_code).map(|r| r.serial.stream_id).unwrap_or_else(Uuid::now_v7)
    };
    let payload = serde_json::json!({
        "station_code": req.station_code,
        "lot_id": req.lot_id,
        "serial_low": req.serial_low,
        "serial_high": req.serial_high,
        "assigned_by": req.assigned_by,
        "assigned_at": chrono::Utc::now().to_rfc3339(),
    });
    if let Err(e) = state.apply(&req.station_code, EVENT_SERIAL_ASSIGNED, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    let record = match state.get(&req.station_code) {
        Some(r) => r,
        None => return problem(StatusCode::INTERNAL_SERVER_ERROR, "assignment was not stored"),
    };
    (
        StatusCode::CREATED,
        Json(AssignSerialResponse {
            assignment_id: stream_id.to_string(),
            station_code: req.station_code,
            serial_low: req.serial_low,
            serial_high: req.serial_high,
            status: AppState::status_wire(&record.serial.status).to_string(),
            assigned_count: record.serial.assigned_count(),
            // The manifest signature is recorded as supplied but not verified:
            // there is no manifest public key reachable from this process.
            blocking_reason: Some(
                "the lot manifest signature is recorded as supplied but not verified here; \
                 verification needs the manifest authority's public key"
                    .to_string(),
            ),
        }),
    )
        .into_response()
}

async fn list_assignments(State(state): State<SharedState>) -> impl IntoResponse {
    let assignments: Vec<serde_json::Value> = state
        .all_sorted()
        .iter()
        .map(|r| {
            serde_json::json!({
                "assignment_id": r.serial.stream_id.to_string(),
                "station_code": r.serial.station_code,
                "lot_id": r.serial.lot_id,
                "serial_low": r.serial.serial_low,
                "serial_high": r.serial.serial_high,
                "assigned_count": r.serial.assigned_count(),
                "status": AppState::status_wire(&r.serial.status),
                "result_hold": r.serial.result_hold,
                "spoilt_serials": r.serial.spoilt_serials,
                "receipt_confirmed": r.receipt_confirmed,
                "blocking_reason": "the lot manifest signature is recorded as supplied but \
                                     not verified here; verification needs the manifest \
                                     authority's public key",
            })
        })
        .collect();
    Json(serde_json::json!({ "assignments": assignments }))
}

async fn confirm_receipt(
    State(state): State<SharedState>,
    Json(req): Json<ReceiptRequest>,
) -> axum::response::Response {
    info!(assignment_id = %req.assignment_id, clerk_id = %req.clerk_id, "ballot-serial: receipt");
    if req.clerk_id.trim().is_empty() || req.receipt_hash.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "clerk_id and receipt_hash are required",
        );
    }
    // Find the station this assignment belongs to. Previously any id was
    // accepted, so a receipt could be confirmed against an assignment that did
    // not exist.
    let Some(record) = state.all_sorted().into_iter().find(|r| r.serial.stream_id.to_string() == req.assignment_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no ballot-serial assignment with id {}", req.assignment_id),
        );
    };
    let station_code = record.serial.station_code.clone();
    let payload = serde_json::json!({
        "clerk_id": req.clerk_id,
        "receipt_hash": req.receipt_hash,
    });
    if let Err(e) = state.apply(&station_code, EVENT_BALLOT_ISSUED, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    state.confirm_receipt(&station_code);
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "receipt_id": Uuid::now_v7().to_string(),
            "assignment_id": req.assignment_id,
            "station_code": station_code,
            "status": "issued",
        })),
    )
        .into_response()
}

async fn record_spoilt(
    State(state): State<SharedState>,
    Json(req): Json<SpoiltRequest>,
) -> axum::response::Response {
    info!(
        station_code = %req.station_code,
        spoilt_serial = req.spoilt_serial,
        "ballot-serial: spoilt"
    );
    let Some(record) = state.get(&req.station_code) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no ballot-serial assignment for station {}", req.station_code),
        );
    };
    // FR-M63-014: a serial must fall inside the range the station was issued.
    // Recording a spoilt ballot outside the range would inflate the spoilt count
    // and mask a real discrepancy.
    if !record.serial.in_range(req.spoilt_serial) {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!(
                "serial {} is outside the range {}..={} assigned to {}",
                req.spoilt_serial,
                record.serial.serial_low,
                record.serial.serial_high,
                req.station_code
            ),
        );
    }
    if record.serial.spoilt_serials.contains(&req.spoilt_serial) {
        return problem(
            StatusCode::CONFLICT,
            format!("serial {} has already been recorded as spoilt", req.spoilt_serial),
        );
    }
    if req.reason.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "reason is required");
    }
    let payload = serde_json::json!({
        "spoilt_serial": req.spoilt_serial,
        "replacement_serial": req.replacement_serial,
        "reason": req.reason,
        "clerk_id": req.clerk_id,
        "spoilt_at": chrono::Utc::now().to_rfc3339(),
    });
    if let Err(e) = state.apply(&req.station_code, EVENT_BALLOT_SPOILT, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    let record = state.get(&req.station_code).unwrap_or(record);
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "station_code": req.station_code,
            "spoilt_serial": req.spoilt_serial,
            "status": "recorded",
            "spoilt_count": record.serial.spoilt_serials.len(),
        })),
    )
        .into_response()
}

async fn trigger_reconciliation(
    State(state): State<SharedState>,
    Json(req): Json<ReconcileRequest>,
) -> axum::response::Response {
    let Some(record) = state.get(&req.station_code) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no ballot-serial assignment for station {}", req.station_code),
        );
    };
    info!(station_code = %req.station_code, "ballot-serial: reconcile");

    // The range is the one the station was issued. Previously this was
    // `1..=deposited + spoilt`, which makes `assigned == accounted` by
    // construction and makes a mismatch impossible to detect.
    let counts = StationCounts {
        station_code: req.station_code.clone(),
        serial_low: record.serial.serial_low,
        serial_high: record.serial.serial_high,
        deposited_count: req.deposited_count,
        spoilt_count: req.spoilt_count,
    };
    // Spoilt serials recorded through this endpoint count toward the spoilt
    // figure. Taking only the request's number would let a station under-report
    // spoilt ballots it had itself recorded.
    let recorded_spoilt = record.serial.spoilt_serials.len() as u64;
    let counts = StationCounts {
        spoilt_count: recorded_spoilt.max(req.spoilt_count),
        ..counts
    };
    let result = reconcile(&counts);
    let is_held = matches!(result.status, ReconciliationStatus::Held { .. });
    state.set_reconciliation(&req.station_code, result.clone());

    let event_type = if is_held {
        EVENT_RECONCILIATION_DISCREPANCY
    } else {
        EVENT_RECONCILIATION_COMPLETE
    };
    let mut payload = serde_json::json!({
        "reconciled_at": chrono::Utc::now().to_rfc3339(),
        "assigned_count": result.assigned_count,
        "deposited_count": result.deposited_count,
        "spoilt_count": result.spoilt_count,
        "unused_count": result.unused_count,
        "discrepancy": result.discrepancy,
    });
    if let Some(reason) = &result.flag_reason {
        payload["flag_reason"] = serde_json::json!(reason);
    }
    if let Err(e) = state.apply(&req.station_code, EVENT_RECONCILIATION_STARTED, payload.clone()) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    if let Err(e) = state.apply(&req.station_code, event_type, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }

    (
        StatusCode::ACCEPTED,
        Json(ReconcileResponse {
            reconciliation_id: Uuid::now_v7().to_string(),
            station_code: req.station_code,
            status: if is_held { "held" } else { "cleared" }.to_string(),
            result_hold: is_held,
            flag_reason: result.flag_reason,
            assigned_count: result.assigned_count,
            deposited_count: result.deposited_count,
            spoilt_count: result.spoilt_count,
            unused_count: result.unused_count,
            discrepancy: result.discrepancy,
        }),
    )
        .into_response()
}

async fn get_reconciliation(
    State(state): State<SharedState>,
    Path(station_id): Path<String>,
) -> axum::response::Response {
    let Some(record) = state.get(&station_id) else {
        // Previously any station code returned `status: "pending",
        // result_hold: false`, so a station that had never been reconciled
        // reported no hold.
        return problem(
            StatusCode::NOT_FOUND,
            format!("no ballot-serial assignment for station {station_id}"),
        );
    };
    Json(serde_json::json!({
        "station_code": station_id,
        "status": AppState::status_wire(&record.serial.status),
        "result_hold": record.serial.result_hold,
        "hold_reason": record.serial.hold_reason,
        "assigned_count": record.serial.assigned_count(),
        "spoilt_count": record.serial.spoilt_serials.len(),
        "reconciliation": record.last_reconciliation,
    }))
    .into_response()
}

async fn resolve_hold(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<ResolveHoldRequest>,
) -> axum::response::Response {
    info!(
        station_code = %id,
        resolved_by_po = %req.resolved_by_po,
        resolved_by_cro = %req.resolved_by_cro,
        "ballot-serial: resolve hold"
    );
    let Some(record) = state.get(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no ballot-serial assignment for station {id}"),
        );
    };
    // Dual control: both officers must sign off. The request has always carried
    // both fields and the handler has always ignored the second one.
    if req.resolved_by_po.trim().is_empty() || req.resolved_by_cro.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "a hold is cleared under dual control; both resolved_by_po and \
             resolved_by_cro are required",
        );
    }
    if req.reason.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "reason is required");
    }
    // There is no hold to clear. Previously this returned `cleared` for any id.
    if !record.serial.result_hold {
        return problem(
            StatusCode::CONFLICT,
            format!("station {id} has no active hold to resolve"),
        );
    }
    let payload = serde_json::json!({
        "resolved_by_po": req.resolved_by_po,
        "resolved_by_cro": req.resolved_by_cro,
        "reason": req.reason,
        "resolved_at": chrono::Utc::now().to_rfc3339(),
    });
    if let Err(e) = state.apply(&id, EVENT_RECONCILIATION_COMPLETE, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "reconciliation_id": Uuid::now_v7().to_string(),
            "station_code": id,
            "status": "cleared",
            "resolved_by_po": req.resolved_by_po,
            "resolved_by_cro": req.resolved_by_cro,
            "resolved_at": chrono::Utc::now().to_rfc3339(),
        })),
    )
        .into_response()
}

// ---- router -----------------------------------------------------------------

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/ballot-serial/assignments", post(assign_serial))
        .route("/api/v1/ballot-serial/assignments", get(list_assignments))
        .route("/api/v1/ballot-serial/receipts", post(confirm_receipt))
        .route("/api/v1/ballot-serial/spoilt", post(record_spoilt))
        .route("/api/v1/ballot-serial/reconcile", post(trigger_reconciliation))
        .route(
            "/api/v1/ballot-serial/reconcile/{station_id}",
            get(get_reconciliation),
        )
        .route("/api/v1/ballot-serial/reconcile/{id}/resolve", put(resolve_hold))
        .with_state(state)
}
