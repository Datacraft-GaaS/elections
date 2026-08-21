use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::models::*;
use crate::saml::SamlBroker;

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
