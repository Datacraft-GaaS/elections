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

//! HTTP server + public contract handlers for M27 intent preservation
//! (support-intent/preservation.yaml).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::spec::{AdjudicationRecord, AdjudicationRequest, PreMarkValidationRequest, SpoiltBallotRequest};

#[derive(Clone, Default)]
pub struct AppState {
    pub adjudications: Arc<Mutex<HashMap<String, Value>>>,
    pub alerts: Arc<Mutex<HashMap<String, Value>>>,
    pub toggle_audit: Arc<Mutex<Vec<Value>>>,
    /// Spoilt ballots as reported, so a submission can be read back.
    ///
    /// `submit_spoilt_ballot` minted a `spoilt_id` and answered 201 without
    /// recording anything, leaving the caller a ticket for a ballot the service had
    /// forgotten.
    pub spoilt_ballots: Arc<Mutex<Vec<Value>>>,
    /// The audit trail, appended as things happen.
    pub audit_log: Arc<Mutex<Vec<Value>>>,
}

impl AppState {
    /// Append an audit entry.
    pub fn audit(&self, kind: &str, subject: &str, detail: &str) {
        self.audit_log.lock().expect("lock poisoned").push(json!({
            "kind": kind,
            "subject": subject,
            "detail": detail,
            "at": chrono::Utc::now().to_rfc3339(),
        }));
    }
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
}
pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/v1/intent/premark", post(submit_pre_mark_validation))
        .route("/v1/intent/spoilt", post(submit_spoilt_ballot))
        .route(
            "/v1/intent/adjudications",
            get(list_adjudications).post(record_adjudication),
        )
        .route("/v1/intent/adjudications/{id}", get(get_adjudication))
        .route("/v1/intent/alerts", get(list_rejection_rate_alerts))
        .route(
            "/v1/intent/alerts/{id}/acknowledge",
            post(acknowledge_alert),
        )
        .route("/v1/intent/catalogue", get(list_adjudication_catalogue))
        .route("/v1/intent/toggle", get(list_toggle_audit).post(toggle_adjudication))
        .with_state(state)
}

async fn submit_pre_mark_validation(Json(req): Json<PreMarkValidationRequest>) -> impl IntoResponse {
    if req.image_delete_elapsed_ms > 1000 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "detail": "image_delete_elapsed_ms must be <= 1000" })),
        )
            .into_response();
    }
    (StatusCode::OK, Json(json!({ "valid": true, "station_code": req.station_code }))).into_response()
}

/// A spoilt ballot, recorded.
///
/// Previously minted a `spoilt_id` and answered 201, leaving the caller a ticket
/// for a spoilt ballot that was never recorded anywhere. A spoilt-ballot record is
/// the evidence a spoiled vote is adjudicated against, so dropping it loses the
/// reason the ballot was spoiled.
async fn submit_spoilt_ballot(
    State(state): State<SharedState>,
    Json(req): Json<SpoiltBallotRequest>,
) -> impl IntoResponse {
    for (field, value) in [
        ("station_code", req.station_code.trim()),
        ("contest_id", req.contest_id.trim()),
        ("reason_code", req.reason_code.trim()),
        ("officer_id", req.officer_id.trim()),
    ] {
        if value.is_empty() {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "detail": format!(
                        "{field} is required: a spoilt ballot is the evidence a spoiled \
                         vote is adjudicated against, and without it the record says \
                         nothing about why the ballot was spoiled"
                    )
                })),
            )
                .into_response();
        }
    }
    // The reason code is checked against the catalogue this crate publishes, so the
    // two endpoints agree: a code that cannot be listed cannot be recorded.
    if let Err(e) = crate::catalogue::lookup(req.reason_code.trim()) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "detail": e.to_string() })),
        )
            .into_response();
    }
    if req.witness_ids.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "detail": "witness_ids is required: a spoilt ballot is recorded against the \
                           officers who witnessed it"
            })),
        )
            .into_response();
    }
    let record = json!({
        "spoilt_id": uuid::Uuid::now_v7().to_string(),
        "station_code": req.station_code.trim(),
        "contest_id": req.contest_id.trim(),
        "reason_code": req.reason_code.trim(),
        "officer_id": req.officer_id.trim(),
        "witness_ids": req.witness_ids,
        "replacement_issued": req.replacement_issued.unwrap_or(false),
        "at": chrono::Utc::now().to_rfc3339(),
    });
    state
        .spoilt_ballots
        .lock()
        .expect("lock poisoned")
        .push(record.clone());
    state.audit(
        "spoilt_ballot_submitted",
        record["spoilt_id"].as_str().unwrap_or_default(),
        record["reason_code"].as_str().unwrap_or_default(),
    );
    (StatusCode::CREATED, Json(record)).into_response()
}

async fn record_adjudication(
    State(state): State<SharedState>,
    Json(req): Json<AdjudicationRequest>,
) -> impl IntoResponse {
    let record = AdjudicationRecord {
        id: uuid::Uuid::now_v7(),
        ballot_id: req.ballot_id.clone(),
        station_code: req.station_code.clone(),
        contest_id: req.contest_id.clone(),
        adjudicator_id: req.adjudicator_id.clone(),
        reason_code: req.reason_code.clone(),
        decision: req.decision.clone(),
        photo_evidence_ref: req.photo_evidence_ref.clone(),
        adjudicated_at: chrono::Utc::now(),
    };
    let id = record.id.to_string();
    let val = serde_json::to_value(&record).unwrap();
    state
        .adjudications
        .lock()
        .expect("lock poisoned")
        .insert(id, val.clone());
    (StatusCode::CREATED, Json(val))
}

async fn list_adjudications(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.adjudications.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "adjudications": items }))
}

async fn get_adjudication(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let guard = state.adjudications.lock().expect("lock poisoned");
    match guard.get(&id) {
        Some(a) => Json(a.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "adjudication not found" }))).into_response(),
    }
}

async fn list_rejection_rate_alerts(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.alerts.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "alerts": items }))
}

async fn acknowledge_alert(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut guard = state.alerts.lock().expect("lock poisoned");
    match guard.get_mut(&id) {
        Some(a) => {
            a["acknowledged"] = json!(true);
            Json(a.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "alert not found" }))).into_response(),
    }
}

/// The standardised adjudication reason codes.
///
/// Previously `{ "catalogue": [] }` for every caller, which claims there are no
/// adjudication reason codes at all. This crate has held them the whole time in
/// `catalogue.rs`: A1-A8 for rejection and V1-V3 for acceptance. An empty
/// catalogue is not a neutral answer on an endpoint an officer picks a code from.
async fn list_adjudication_catalogue() -> impl IntoResponse {
    Json(json!({ "catalogue": crate::catalogue::CATALOGUE }))
}

async fn toggle_adjudication(
    State(state): State<SharedState>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let record = json!({ "toggle_id": uuid::Uuid::now_v7().to_string(), "request": req, "at": chrono::Utc::now().to_rfc3339() });
    state
        .toggle_audit
        .lock()
        .expect("lock poisoned")
        .push(record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn list_toggle_audit(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.toggle_audit.lock().expect("lock poisoned");
    let items = guard.clone();
    drop(guard);
    Json(json!({ "toggle_audit": items }))
}
