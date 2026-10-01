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

/// A declared operation with no implementation behind it.
///
/// `sign_mou` and `resolve_incident` are why this crate needs it. Signing an MOU
/// between a county and a ministry, and resolving an election-security incident,
/// are both attestations: the response is the only evidence the caller has that
/// the thing happened. Answering 204 for either means the record exists nowhere
/// and the caller cannot tell.
fn not_implemented(operation: &str, detail: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": "Not Implemented",
            "status": 501,
            "detail": format!(
                "{operation} is declared by the contract but has no \
                 implementation behind it: {detail}"
            ),
        })),
    )
}

pub async fn list_mous() -> impl IntoResponse {
    info!("list_mous");
    not_implemented(
        "MOU listing",
        "no MOU store is wired; it previously returned an empty list",
    )
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
    // The body above echoes the request into a draft and answers 201, storing
    // nothing.
    let _ = &mou;
    not_implemented(
        "MOU creation",
        "no MOU store is wired; it previously returned 201 with the request \
         echoed into a draft record",
    )
}

pub async fn sign_mou(Path(id): Path<String>, Json(_req): Json<SignMouRequest>) -> impl IntoResponse {
    info!(id, "sign_mou");
    // Logged the signing officer's id and answered 204. A 204 is a successful
    // signature: the caller had no other evidence the MOU was signed, and none
    // existed anywhere.
    let _ = id;
    not_implemented(
        "MOU signing",
        "no MOU store is wired; it previously returned 204 for a signature \
         that was recorded nowhere",
    )
}

pub async fn list_datasets() -> impl IntoResponse {
    info!("list_datasets");
    not_implemented(
        "dataset listing",
        "no dataset store is wired; it previously returned an empty list",
    )
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
    // Echoed the submission into a dataset record and answered 201, storing
    // nothing. approved_by and approval_date are already None here, so the
    // record never claimed approval -- but the 201 claimed receipt.
    let _ = &dataset;
    not_implemented(
        "dataset submission",
        "no dataset store is wired; it previously returned 201 with the \
         submission echoed into a record it never stored",
    )
}

pub async fn list_incidents() -> impl IntoResponse {
    info!("list_incidents");
    // An empty incident list reads as "no incidents are open", which is a
    // materially different claim from "nothing is stored".
    not_implemented(
        "incident listing",
        "no incident store is wired; it previously returned an empty list, \
         which reads as no incidents being open",
    )
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
    // Discarded the resolution notes the caller supplied and answered 204.
    // Resolving an election-security incident is an attestation that the
    // incident is closed; the only record of the resolution was the request
    // that is now gone.
    let _ = (id, &req);
    not_implemented(
        "incident resolution",
        "no incident store is wired; it discarded the resolution notes and \
         returned 204 for a resolution recorded nowhere",
    )
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
