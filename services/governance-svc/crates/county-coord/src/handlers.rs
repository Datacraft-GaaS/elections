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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::saml::SamlBroker;
use crate::spec;
use crate::state::SharedState;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}


pub async fn list_mous(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_mous");
    Json(serde_json::json!({ "mous": state.mous.list() }))
}

pub async fn create_mou(
    State(state): State<SharedState>,
    Json(req): Json<CreateMouRequest>,
) -> impl IntoResponse {
    info!(county_code = %req.county_code, "create_mou");
    let mou = Mou {
        id: uuid::Uuid::new_v4().to_string(),
        county_code: req.county_code,
        ministry_code: req.ministry_code,
        party_type: req.party_type,
        effective_date: req.effective_date,
        expiry_date: req.expiry_date,
        content_url: req.content_url,
        status: MouStatus::Draft,
        signed_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    // Real. Previously echoed the request into a draft and answered 201 while
    // storing nothing, so it never appeared in list_mous and a re-read never
    // found it.
    state.mous.insert(mou.id.clone(), mou.clone());
    (StatusCode::CREATED, Json(mou))
}

pub async fn sign_mou(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<SignMouRequest>,
) -> impl IntoResponse {
    // The signer and their signature hash are the evidence a signature exists,
    // and the Mou model has no field for either, so they are recorded in the
    // audit log rather than dropped. Previously officer_id reached only a log
    // line and the response was 204 for any id.
    info!(
        id,
        officer_id = %req.officer_id,
        signature_hash = %req.signature_hash,
        "sign_mou"
    );
    let signed_at = chrono::Utc::now().to_rfc3339();
    match state.mous.update(&id, |mut m| {
        m.status = MouStatus::Signed;
        m.signed_at = Some(signed_at.clone());
        m
    }) {
        Some(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        // Previously answered 204 for any id at all, so signing a MOU that does
        // not exist was indistinguishable from signing one that did.
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "type": "about:blank",
                "title": "Not Found",
                "status": 404,
                "detail": format!("no MOU with id {id}"),
            })),
        ),
    }
    .into_response()
}

pub async fn list_datasets(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_datasets");
    Json(serde_json::json!({ "datasets": state.datasets.list() }))
}

pub async fn submit_dataset(
    State(state): State<SharedState>,
    Json(req): Json<SubmitDatasetRequest>,
) -> impl IntoResponse {
    info!(county_code = %req.county_code, dataset_type = ?req.dataset_type, "submit_dataset");
    let dataset = CountyDataset {
        id: uuid::Uuid::new_v4().to_string(),
        county_code: req.county_code,
        dataset_type: req.dataset_type,
        source_system: req.source_system,
        approved_by: None,
        approval_date: None,
        dataset_url: req.dataset_url,
        schema_version: req.schema_version,
        lawful_basis_code: req.lawful_basis_code,
        purpose_code: req.purpose_code,
        retention_until: req.retention_until,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    state.datasets.insert(dataset.id.clone(), dataset.clone());
    (StatusCode::CREATED, Json(dataset))
}

pub async fn list_incidents(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_incidents");
    // Real: previously always an empty list, which asserted that no incidents
    // were open rather than that nothing was stored.
    Json(serde_json::json!({ "incidents": state.incidents.list() }))
}

pub async fn create_incident(
    State(state): State<SharedState>,
    Json(req): Json<CreateIncidentRequest>,
) -> impl IntoResponse {
    info!(
        county_code = %req.county_code,
        severity = ?req.severity,
        "create_incident"
    );
    let incident = CoordinationIncident {
        id: uuid::Uuid::new_v4().to_string(),
        county_code: req.county_code,
        incident_type: req.incident_type,
        severity: req.severity,
        description: req.description,
        reported_at: chrono::Utc::now().to_rfc3339(),
        resolved_at: None,
        ecc_alert_ref: None,
    };
    state.incidents.insert(incident.id.clone(), incident.clone());
    (StatusCode::CREATED, Json(incident))
}

pub async fn resolve_incident(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<ResolveIncidentRequest>,
) -> impl IntoResponse {
    info!(id, "resolve_incident");
    // The notes are the substance of this call. They used to be discarded
    // (`let _ = req.resolution_notes`) and the handler answered 204 for any id,
    // so a caller resolving an incident that never existed was told it had been
    // resolved. The model has no field for the notes, so they are acknowledged
    // in the audit log rather than silently dropped.
    info!(resolution_notes = %req.resolution_notes, "incident resolved");
    let resolved_at = chrono::Utc::now().to_rfc3339();
    match state.incidents.update(&id, |mut i| {
        i.resolved_at = Some(resolved_at.clone());
        i
    }) {
        Some(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "type": "about:blank",
                "title": "Not Found",
                "status": 404,
                "detail": format!("no incident with id {id}"),
            })),
        ),
    }
}

pub async fn saml_acs(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let county_code = body
        .get("county_code")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let assertion_xml = body
        .get("assertion_xml")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    info!(county_code, "saml_acs");
    let broker = SamlBroker;
    match broker.process_acs(county_code, assertion_xml) {
        Ok(token) => (StatusCode::OK, Json(serde_json::json!({ "session_token": token }))),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": e })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-county-coord/coordination.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/county-coord/status-reports — submit a county status
/// report.
pub async fn submit_county_status_report(
    State(state): State<SharedState>,
    Json(req): Json<spec::CountyStatusReportRequest>,
) -> impl IntoResponse {
    info!(county = %req.county_code, "submit_county_status_report");
    let report = spec::CountyStatusReport {
        report_id: uuid::Uuid::now_v7().to_string(),
        county_code: req.county_code,
        stations_open: req.stations_open,
        stations_closed_early: req.stations_closed_early,
        incidents: req.incidents,
        notes: req.notes,
        submitted_at: req.submitted_at,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.reports.lock().expect("lock poisoned");
    guard.insert(report.report_id.clone(), report.clone());
    drop(guard);
    (StatusCode::CREATED, Json(report)).into_response()
}

/// GET /api/v1/governance/county-coord/status-reports — list county status
/// reports.
pub async fn list_county_status_reports(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_county_status_reports");
    let guard = state.reports.lock().expect("lock poisoned");
    let mut items: Vec<spec::CountyStatusReport> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.submitted_at.cmp(&b.submitted_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// POST /api/v1/governance/county-coord/escalations — escalate an incident.
pub async fn escalate_to_national(
    State(state): State<SharedState>,
    Json(req): Json<spec::CoordinationEscalationRequest>,
) -> impl IntoResponse {
    info!(county = %req.county_code, severity = %req.severity, "escalate_to_national");
    let now = chrono::Utc::now().to_rfc3339();
    let escalation = spec::CoordinationEscalation {
        escalation_id: uuid::Uuid::now_v7().to_string(),
        county_code: req.county_code,
        incident_type: req.incident_type,
        severity: req.severity,
        description: req.description,
        status: "open".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.escalations.lock().expect("lock poisoned");
    guard.insert(escalation.escalation_id.clone(), escalation.clone());
    drop(guard);
    (StatusCode::CREATED, Json(escalation)).into_response()
}

/// GET /api/v1/governance/county-coord/escalations — list escalations.
pub async fn list_escalations(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_escalations");
    let guard = state.escalations.lock().expect("lock poisoned");
    let mut items: Vec<spec::CoordinationEscalation> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}
