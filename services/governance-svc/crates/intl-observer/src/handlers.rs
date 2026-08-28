use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;
use crate::models::*;
use crate::spec;
use crate::state::SharedState;

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

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-intl-observer/coordination.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/intl-observer/missions — register an observer mission.
pub async fn register_observer_mission(
    State(state): State<SharedState>,
    Json(req): Json<spec::ObserverMissionRequest>,
) -> impl IntoResponse {
    info!(mission_name = %req.mission_name, "register_observer_mission");
    let now = chrono::Utc::now().to_rfc3339();
    let mission = spec::ObserverMission {
        mission_id: uuid::Uuid::now_v7().to_string(),
        mission_name: req.mission_name,
        organisation: req.organisation,
        lead_observer_id_hash: req.lead_observer_id_hash,
        country_of_origin: req.country_of_origin,
        observer_count: req.observer_count,
        accreditation_ref: req.accreditation_ref,
        status: "registered".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.missions.lock().expect("lock poisoned");
    guard.insert(mission.mission_id.clone(), mission.clone());
    drop(guard);
    (StatusCode::CREATED, Json(mission)).into_response()
}

/// GET /api/v1/governance/intl-observer/missions — list observer missions.
pub async fn list_observer_missions(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_observer_missions");
    let guard = state.missions.lock().expect("lock poisoned");
    let mut items: Vec<spec::ObserverMission> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// GET /api/v1/governance/intl-observer/missions/{mission_id} — fetch one mission.
pub async fn get_observer_mission(
    State(state): State<SharedState>,
    Path(mission_id): Path<String>,
) -> impl IntoResponse {
    info!(mission_id, "get_observer_mission");
    let guard = state.missions.lock().expect("lock poisoned");
    match guard.get(&mission_id) {
        Some(m) => Json(m.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "mission not found"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/governance/intl-observer/missions/{mission_id}/reports — submit an
/// observer report.
pub async fn submit_observer_report(
    State(state): State<SharedState>,
    Path(mission_id): Path<String>,
    Json(req): Json<spec::ObserverReportRequest>,
) -> impl IntoResponse {
    info!(mission_id, "submit_observer_report");
    if !state.missions.lock().expect("lock poisoned").contains_key(&mission_id) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "mission not found"})),
        )
            .into_response();
    }
    let report = spec::ObserverReport {
        report_id: uuid::Uuid::now_v7().to_string(),
        mission_id: mission_id.clone(),
        report_type: req.report_type,
        findings: req.findings,
        recommendations: req.recommendations,
        evidence_refs: req.evidence_refs,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.reports.lock().expect("lock poisoned");
    guard.entry(mission_id).or_default().push(report.clone());
    drop(guard);
    (StatusCode::CREATED, Json(report)).into_response()
}

/// GET /api/v1/governance/intl-observer/missions/{mission_id}/reports — list
/// observer reports.
pub async fn list_observer_reports(
    State(state): State<SharedState>,
    Path(mission_id): Path<String>,
) -> impl IntoResponse {
    info!(mission_id, "list_observer_reports");
    let guard = state.reports.lock().expect("lock poisoned");
    let items = guard.get(&mission_id).cloned().unwrap_or_default();
    drop(guard);
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}
