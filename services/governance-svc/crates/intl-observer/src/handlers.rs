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

use crate::models::*;
use crate::spec;
use crate::state::SharedState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

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

pub async fn list_missions(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_missions");
    Json(serde_json::json!({ "missions": state.legacy_missions_sorted() }))
}

pub async fn register_mission(
    State(state): State<SharedState>,
    Json(req): Json<RegisterMissionRequest>,
) -> axum::response::Response {
    info!(org = %req.organisation_name, "register_mission");
    if req.organisation_name.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "organisation_name is required",
        );
    }
    let id = uuid::Uuid::now_v7().to_string();
    // The workspace id is derived from the mission id by the engine. Previously
    // this was the literal "ws-stub" for every mission, so two missions shared
    // one workspace reference.
    let accredited = !req.accreditation_ref.trim().is_empty();
    let mission = ObserverMission {
        id: id.clone(),
        organisation_name: req.organisation_name,
        accreditation_ref: req.accreditation_ref,
        lead_contact_hash: req.lead_contact_hash,
        workspace_id: crate::engine::ObserverEngine::provision_workspace(&id),
        // Accreditation is what entitles a mission to observe, so an accredited
        // registration is an active mission. An unaccredited one is merely
        // registered. This is the only path to `Active`, which matters because
        // `can_submit_report` gates on it -- without it `submit_report` would be
        // permanently closed.
        status: if accredited {
            MissionStatus::Active
        } else {
            MissionStatus::Registered
        },
        registered_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .legacy_missions
        .lock()
        .expect("lock poisoned")
        .insert(id, mission.clone());
    (StatusCode::CREATED, Json(mission)).into_response()
}

/// Look up a mission, or answer 404. Every `/v1` endpoint that names a mission
/// goes through this, so a briefing or report can never be attached to a
/// mission that was never registered.
// axum::response::Response is a large Err variant and is the natural error type
// for a handler; boxing it would obscure the error path for no benefit.
#[allow(clippy::result_large_err)]
fn require_mission(
    state: &SharedState,
    mission_id: &str,
) -> Result<crate::models::ObserverMission, axum::response::Response> {
    let guard = state.legacy_missions.lock().expect("lock poisoned");
    match guard.get(mission_id) {
        Some(m) => Ok(m.clone()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no observer mission with id {mission_id}"),
        )),
    }
}

pub async fn list_briefings(
    State(state): State<SharedState>,
    Path(mission_id): Path<String>,
) -> axum::response::Response {
    info!(mission_id, "list_briefings");
    if let Err(resp) = require_mission(&state, &mission_id) {
        return resp;
    }
    Json(serde_json::json!({ "briefings": state.briefings_for(&mission_id) })).into_response()
}

pub async fn schedule_briefing(
    State(state): State<SharedState>,
    Path(mission_id): Path<String>,
    Json(req): Json<ScheduleBriefingRequest>,
) -> axum::response::Response {
    info!(mission_id, topic = %req.topic, "schedule_briefing");
    if let Err(resp) = require_mission(&state, &mission_id) {
        return resp;
    }
    if req.topic.trim().is_empty() || req.scheduled_at.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "topic and scheduled_at are required",
        );
    }
    let briefing = Briefing {
        id: uuid::Uuid::now_v7().to_string(),
        mission_id: mission_id.clone(),
        topic: req.topic,
        scheduled_at: req.scheduled_at,
        materials_url: req.materials_url,
        // Attendance is confirmed by the mission, never by the scheduling act.
        attendance_confirmed: false,
    };
    state
        .briefings
        .lock()
        .expect("lock poisoned")
        .entry(mission_id)
        .or_default()
        .push(briefing.clone());
    (StatusCode::CREATED, Json(briefing)).into_response()
}

pub async fn submit_report(
    State(state): State<SharedState>,
    Path(mission_id): Path<String>,
    Json(req): Json<SubmitReportRequest>,
) -> axum::response::Response {
    info!(mission_id, "submit_report");
    let Ok(mission) = require_mission(&state, &mission_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no observer mission with id {mission_id}"),
        );
    };
    // The engine encoded this rule -- only an active mission may report -- and
    // the handler never called it, so a mission that had only ever been
    // registered could file observations.
    if !crate::engine::ObserverEngine::can_submit_report(&mission) {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "mission is {}; only an active mission may file an observation report",
                mission.status.wire()
            ),
        );
    }
    let report = ObservationReport {
        id: uuid::Uuid::now_v7().to_string(),
        mission_id,
        report_type: req.report_type,
        submitted_at: chrono::Utc::now().to_rfc3339(),
        content_url: req.content_url,
        classified: req.classified,
    };
    state
        .observation_reports
        .lock()
        .expect("lock poisoned")
        .entry(report.mission_id.clone())
        .or_default()
        .push(report.clone());
    (StatusCode::CREATED, Json(report)).into_response()
}

pub async fn list_reports(
    State(state): State<SharedState>,
    Path(mission_id): Path<String>,
) -> axum::response::Response {
    info!(mission_id, "list_reports");
    if let Err(resp) = require_mission(&state, &mission_id) {
        return resp;
    }
    Json(serde_json::json!({ "reports": state.observation_reports_for(&mission_id) }))
        .into_response()
}

pub async fn publish_joint_statement(
    State(state): State<SharedState>,
    Json(req): Json<PublishJointStatementRequest>,
) -> axum::response::Response {
    info!(
        mission_count = req.participating_mission_ids.len(),
        "publish_joint_statement"
    );
    if !crate::engine::ObserverEngine::validate_joint_statement_participants(
        &req.participating_mission_ids,
    ) {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "a joint statement needs at least one participating mission",
        );
    }
    // The engine's check is only "not empty" -- it cannot see the store. A
    // statement attributing itself to missions that do not exist is worse than
    // no statement, so every participant is resolved.
    let mut missing: Vec<String> = Vec::new();
    {
        let guard = state.legacy_missions.lock().expect("lock poisoned");
        for id in &req.participating_mission_ids {
            if !guard.contains_key(id) {
                missing.push(id.clone());
            }
        }
    }
    if !missing.is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("unknown participating missions: {}", missing.join(", ")),
        );
    }
    let stmt = JointStatement {
        id: uuid::Uuid::now_v7().to_string(),
        participating_mission_ids: req.participating_mission_ids,
        published_at: chrono::Utc::now().to_rfc3339(),
        content_url: req.content_url,
        signed_by: req.signed_by,
    };
    state
        .joint_statements
        .lock()
        .expect("lock poisoned")
        .insert(stmt.id.clone(), stmt.clone());
    (StatusCode::CREATED, Json(stmt)).into_response()
}

pub async fn list_joint_statements(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_joint_statements");
    Json(serde_json::json!({ "joint_statements": state.joint_statements_sorted() }))
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
    Json(
        serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }),
    )
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
    if !state
        .missions
        .lock()
        .expect("lock poisoned")
        .contains_key(&mission_id)
    {
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
    Json(
        serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }),
    )
}
