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

pub async fn list_mous() -> impl IntoResponse {
    info!("list_mous");
    let mous: Vec<Mou> = vec![];
    Json(serde_json::json!({ "mous": mous }))
}

pub async fn create_mou(Json(req): Json<CreateMouRequest>) -> impl IntoResponse {
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
    (StatusCode::CREATED, Json(mou))
}

pub async fn sign_mou(Path(id): Path<String>, Json(req): Json<SignMouRequest>) -> impl IntoResponse {
    info!(id, officer_id = %req.officer_id, "sign_mou");
    StatusCode::NO_CONTENT
}

pub async fn list_datasets() -> impl IntoResponse {
    info!("list_datasets");
    let datasets: Vec<CountyDataset> = vec![];
    Json(serde_json::json!({ "datasets": datasets }))
}

pub async fn submit_dataset(Json(req): Json<SubmitDatasetRequest>) -> impl IntoResponse {
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
    (StatusCode::CREATED, Json(dataset))
}

pub async fn list_incidents() -> impl IntoResponse {
    info!("list_incidents");
    let incidents: Vec<CoordinationIncident> = vec![];
    Json(serde_json::json!({ "incidents": incidents }))
}

pub async fn create_incident(Json(req): Json<CreateIncidentRequest>) -> impl IntoResponse {
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
    (StatusCode::CREATED, Json(incident))
}

pub async fn resolve_incident(
    Path(id): Path<String>,
    Json(req): Json<ResolveIncidentRequest>,
) -> impl IntoResponse {
    info!(id, "resolve_incident");
    let _ = req.resolution_notes;
    StatusCode::NO_CONTENT
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
