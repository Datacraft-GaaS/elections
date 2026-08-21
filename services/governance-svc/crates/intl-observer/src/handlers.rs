use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;
use crate::models::*;

pub async fn health() -> impl IntoResponse { StatusCode::OK }

pub async fn list_missions() -> impl IntoResponse {
    info!("list_missions");
    let missions: Vec<ObserverMission> = vec![];
    Json(serde_json::json!({ "missions": missions }))
}

pub async fn register_mission(Json(req): Json<RegisterMissionRequest>) -> impl IntoResponse {
    info!(org = %req.organisation_name, "register_mission");
    let mission = ObserverMission {
        id: uuid::Uuid::new_v4().to_string(),
        organisation_name: req.organisation_name,
        accreditation_ref: req.accreditation_ref,
        lead_contact_hash: req.lead_contact_hash,
        workspace_id: "ws-stub".to_string(),
        status: MissionStatus::Registered,
        registered_at: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::CREATED, Json(mission))
}

pub async fn list_briefings(Path(mission_id): Path<String>) -> impl IntoResponse {
    info!(mission_id, "list_briefings");
    let briefings: Vec<Briefing> = vec![];
    Json(serde_json::json!({ "briefings": briefings }))
}

pub async fn schedule_briefing(
    Path(mission_id): Path<String>,
    Json(req): Json<ScheduleBriefingRequest>,
) -> impl IntoResponse {
    info!(mission_id, topic = %req.topic, "schedule_briefing");
    let briefing = Briefing {
        id: uuid::Uuid::new_v4().to_string(),
        mission_id,
        topic: req.topic,
        scheduled_at: req.scheduled_at,
        materials_url: req.materials_url,
        attendance_confirmed: false,
    };
    (StatusCode::CREATED, Json(briefing))
}

pub async fn submit_report(
    Path(mission_id): Path<String>,
    Json(req): Json<SubmitReportRequest>,
) -> impl IntoResponse {
    info!(mission_id, "submit_report");
    let report = ObservationReport {
        id: uuid::Uuid::new_v4().to_string(),
        mission_id,
        report_type: req.report_type,
        submitted_at: chrono::Utc::now().to_rfc3339(),
        content_url: req.content_url,
        classified: req.classified,
    };
    (StatusCode::CREATED, Json(report))
}

pub async fn list_reports(Path(mission_id): Path<String>) -> impl IntoResponse {
    info!(mission_id, "list_reports");
    let reports: Vec<ObservationReport> = vec![];
    Json(serde_json::json!({ "reports": reports }))
}

pub async fn publish_joint_statement(Json(req): Json<PublishJointStatementRequest>) -> impl IntoResponse {
    info!(mission_count = req.participating_mission_ids.len(), "publish_joint_statement");
    let stmt = JointStatement {
        id: uuid::Uuid::new_v4().to_string(),
        participating_mission_ids: req.participating_mission_ids,
        published_at: chrono::Utc::now().to_rfc3339(),
        content_url: req.content_url,
        signed_by: req.signed_by,
    };
    (StatusCode::CREATED, Json(stmt))
}

pub async fn list_joint_statements() -> impl IntoResponse {
    info!("list_joint_statements");
    let stmts: Vec<JointStatement> = vec![];
    Json(serde_json::json!({ "joint_statements": stmts }))
}
