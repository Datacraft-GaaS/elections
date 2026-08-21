use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::info;
use uuid::Uuid;
use crate::models::*;

pub async fn report_incident(Json(req): Json<MedicalIncidentReq>) -> impl IntoResponse {
    info!(station_id=%req.station_id, incident_type=%req.incident_type, severity=%req.severity, "report_incident");
    (StatusCode::CREATED, Json(MedicalIncident {
        id: Uuid::new_v4(),
        station_id: req.station_id,
        incident_type: req.incident_type,
        severity: req.severity,
        status: "open".to_string(),
        reported_at: "2027-08-09T06:00:00Z".to_string(),
        persons_affected: req.persons_affected,
    }))
}

pub async fn list_incidents() -> impl IntoResponse {
    Json(serde_json::json!({ "incidents": [] }))
}

pub async fn dispatch_incident(
    Path(id): Path<Uuid>,
    Json(req): Json<DispatchReq>,
) -> impl IntoResponse {
    info!(incident_id=%id, dispatched_to=%req.dispatched_to, "dispatch_incident");
    Json(serde_json::json!({ "incident_id": id, "dispatched_to": req.dispatched_to, "status": "dispatched" }))
}

pub async fn resolve_incident(Path(id): Path<Uuid>) -> impl IntoResponse {
    info!(incident_id=%id, "resolve_incident");
    Json(serde_json::json!({ "incident_id": id, "status": "resolved" }))
}

pub async fn list_cluster_alerts() -> impl IntoResponse {
    Json(serde_json::json!({ "alerts": [] }))
}

pub async fn resolve_cluster_alert(Path(id): Path<Uuid>) -> impl IntoResponse {
    info!(alert_id=%id, "resolve_cluster_alert");
    Json(serde_json::json!({ "alert_id": id, "status": "resolved" }))
}
