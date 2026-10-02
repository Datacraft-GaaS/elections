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

//! HTTP server + public contract handlers for M25 special-voter workflows
//! (support-special-voter/workflows.yaml).

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
use uuid::Uuid;

use crate::spec::{EventRecord, EventRequest, Facility, Form34Request, IdpReassignmentRequest};

#[derive(Clone, Default)]
pub struct AppState {
    pub facilities: Arc<Mutex<HashMap<Uuid, Facility>>>,
    pub events: Arc<Mutex<HashMap<Uuid, Vec<EventRecord>>>>,
    pub forms: Arc<Mutex<HashMap<Uuid, Value>>>,
    pub idp_reassignments: Arc<Mutex<Vec<Value>>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new_shared() -> SharedState {
        Arc::new(Self::default())
    }
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/v1/special/facilities", get(list_special_facilities))
        .route("/v1/special/facilities/{id}", get(get_special_facility))
        .route(
            "/v1/special/facilities/{id}/events",
            get(list_special_voter_events).post(submit_special_voter_event),
        )
        .route(
            "/v1/special/forms/34",
            get(list_special_forms34).post(submit_special_form34),
        )
        .route("/v1/special/forms/34/{id}", get(get_special_form34))
        .route(
            "/v1/special/idp/reassignments",
            post(record_idp_reassignment),
        )
        .route("/v1/special/dashboard", get(get_special_dashboard))
        .route(
            "/v1/special/audit/sample/{category}",
            get(get_audit_sample),
        )
        .route("/v1/special/public/counts", get(get_public_voter_counts))
        .with_state(state)
}

async fn list_special_facilities(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.facilities.lock().expect("lock poisoned");
    let mut items: Vec<Facility> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.id.cmp(&b.id));
    Json(json!({ "facilities": items }))
}

async fn get_special_facility(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let guard = state.facilities.lock().expect("lock poisoned");
    match guard.get(&id) {
        Some(f) => Json(f.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "facility not found" }))).into_response(),
    }
}

async fn submit_special_voter_event(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(req): Json<EventRequest>,
) -> impl IntoResponse {
    let record = EventRecord {
        id: Uuid::now_v7(),
        facility_id: id,
        event_type: req.event_type,
        officer_id: req.officer_id,
        witness_ids: req.witness_ids,
        reason_code: req.reason_code,
        merkle_node_id: None,
        event_at: chrono::Utc::now(),
    };
    state
        .events
        .lock()
        .expect("lock poisoned")
        .entry(id)
        .or_default()
        .push(record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn list_special_voter_events(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let guard = state.events.lock().expect("lock poisoned");
    let items = guard.get(&id).cloned().unwrap_or_default();
    drop(guard);
    Json(json!({ "events": items }))
}

async fn submit_special_form34(
    State(state): State<SharedState>,
    Json(req): Json<Form34Request>,
) -> impl IntoResponse {
    let id = req.facility_id;
    let mut form = serde_json::to_value(&req).unwrap();
    form["submitted_at"] = json!(chrono::Utc::now().to_rfc3339());
    state
        .forms
        .lock()
        .expect("lock poisoned")
        .insert(id, form.clone());
    (StatusCode::CREATED, Json(form))
}

async fn list_special_forms34(State(state): State<SharedState>) -> impl IntoResponse {
    let guard = state.forms.lock().expect("lock poisoned");
    let items: Vec<Value> = guard.values().cloned().collect();
    drop(guard);
    Json(json!({ "forms": items }))
}

async fn get_special_form34(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let guard = state.forms.lock().expect("lock poisoned");
    match guard.get(&id) {
        Some(f) => Json(f.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "detail": "form not found" }))).into_response(),
    }
}

async fn record_idp_reassignment(
    State(state): State<SharedState>,
    Json(req): Json<IdpReassignmentRequest>,
) -> impl IntoResponse {
    let record = json!({
        "reassignment_id": uuid::Uuid::now_v7().to_string(),
        "voter_id_enc": req.voter_id_enc,
        "idp_facility_id": req.idp_facility_id.to_string(),
        "displacement_source": req.displacement_source,
        "recorded_at": chrono::Utc::now().to_rfc3339(),
    });
    state
        .idp_reassignments
        .lock()
        .expect("lock poisoned")
        .push(record.clone());
    (StatusCode::CREATED, Json(record))
}

async fn get_special_dashboard(State(state): State<SharedState>) -> impl IntoResponse {
    let facilities = state.facilities.lock().expect("lock poisoned").len();
    Json(json!({
        "facilities_count": facilities,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn get_audit_sample(
    State(state): State<SharedState>,
    Path(category): Path<String>,
) -> impl IntoResponse {
    let guard = state.facilities.lock().expect("lock poisoned");
    let sample: Vec<Facility> = guard
        .values()
        .filter(|f| f.category == category)
        .take(20)
        .cloned()
        .collect();
    drop(guard);
    Json(json!({ "category": category, "sample": sample }))
}

async fn get_public_voter_counts(State(state): State<SharedState>) -> impl IntoResponse {
    let facilities = state.facilities.lock().expect("lock poisoned").len();
    Json(json!({
        "total_facilities": facilities,
        "as_of": chrono::Utc::now().to_rfc3339(),
    }))
}
