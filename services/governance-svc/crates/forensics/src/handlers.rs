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
use crate::spec;
use crate::state::SharedState;

pub async fn health() -> impl IntoResponse { StatusCode::OK }

/// Cycle status transitions are enforced: only a draft cycle may start a run,
/// and only a complete cycle may publish a replication package or a report.
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

pub async fn list_cycles(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_cycles");
    Json(serde_json::json!({ "cycles": state.cycles_sorted() }))
}

pub async fn create_cycle(
    State(state): State<SharedState>,
    Json(req): Json<CreateCycleRequest>,
) -> axum::response::Response {
    if req.election_id.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "election_id is required");
    }
    let now = chrono::Utc::now().to_rfc3339();
    let cycle = AnalysisCycle {
        id: uuid::Uuid::now_v7().to_string(),
        election_id: req.election_id,
        method: req.method,
        status: CycleStatus::Draft,
        created_at: now,
        ..Default::default()
    };
    state
        .cycles
        .lock()
        .expect("lock poisoned")
        .insert(cycle.id.clone(), cycle.clone());
    (StatusCode::CREATED, Json(cycle)).into_response()
}

pub async fn trigger_run(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    info!(cycle_id = %id, "trigger_run");
    let mut guard = state.cycles.lock().expect("lock poisoned");
    let Some(cycle) = guard.get_mut(&id) else {
        return problem(StatusCode::NOT_FOUND, format!("no cycle with id {id}"));
    };
    if cycle.status != CycleStatus::Draft {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "cycle is {}; only a draft cycle can be started",
                cycle.status.wire()
            ),
        );
    }
    crate::engine::ForensicsEngine::start_run(cycle);
    // The estimator is a separate Python distribution with no bridge in this
    // workspace, so the cycle is recorded as blocked rather than being given a
    // fabricated estimate. See `AnalysisCycle::blocked_reason`.
    cycle.blocked_reason = Some(format!(
        "the {} estimator lives in services/governance-svc/py/forensics-ml, which has no \
         in-process or IPC bridge in this workspace",
        cycle.method.wire()
    ));
    let cycle = cycle.clone();
    drop(guard);
    (StatusCode::ACCEPTED, Json(cycle)).into_response()
}

pub async fn list_estimates(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    info!(cycle_id = %id, "list_estimates");
    if !state.cycles.lock().expect("lock poisoned").contains_key(&id) {
        return problem(StatusCode::NOT_FOUND, format!("no cycle with id {id}"));
    }
    Json(serde_json::json!({ "estimates": state.estimates_for(&id) })).into_response()
}

pub async fn create_referral(
    State(state): State<SharedState>,
    Path(cycle_id): Path<String>,
    Json(req): Json<CreateReferralRequest>,
) -> axum::response::Response {
    info!(cycle_id, referred_to = ?req.referred_to, "create_referral");
    let guard = state.cycles.lock().expect("lock poisoned");
    let Some(cycle) = guard.get(&cycle_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no cycle with id {cycle_id}"),
        );
    };
    // An ODPP referral is built by the engine so the content reference and id
    // follow one convention rather than two.
    let basis = req.referral_basis;
    let mut packet = if req.referred_to == ReferralTarget::Odpp {
        // The engine owns the id and content-reference convention for ODPP.
        crate::engine::ForensicsEngine::build_odpp_packet(cycle, &basis)
    } else {
        ReferralPacket {
            id: uuid::Uuid::now_v7().to_string(),
            cycle_id: cycle_id.clone(),
            referred_to: req.referred_to,
            referral_basis: basis,
            content_url: String::new(),
            referred_at: chrono::Utc::now().to_rfc3339(),
            acknowledged_at: None,
        }
    };
    // The caller's content reference always wins: the engine's is a fallback
    // convention, not a reason to discard what was actually supplied.
    if !req.content_url.is_empty() {
        packet.content_url = req.content_url;
    }
    drop(guard);
    state
        .referrals
        .lock()
        .expect("lock poisoned")
        .insert(packet.id.clone(), packet.clone());
    (StatusCode::CREATED, Json(packet)).into_response()
}

pub async fn publish_replication(
    State(state): State<SharedState>,
    Path(cycle_id): Path<String>,
    Json(req): Json<PublishReplicationRequest>,
) -> axum::response::Response {
    info!(cycle_id, "publish_replication");
    let guard = state.cycles.lock().expect("lock poisoned");
    let Some(cycle) = guard.get(&cycle_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no cycle with id {cycle_id}"),
        );
    };
    // The engine already encodes this rule; publishing against a draft cycle
    // would attest to a result that has not been produced.
    if !crate::engine::ForensicsEngine::can_publish_replication(cycle) {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "cycle is {}; replication requires a complete cycle",
                cycle.status.wire()
            ),
        );
    }
    let pkg = ReplicationPackage {
        id: uuid::Uuid::now_v7().to_string(),
        cycle_id,
        code_url: req.code_url,
        data_hash: req.data_hash,
        environment_spec: req.environment_spec,
        doi: req.doi,
        published_at: chrono::Utc::now().to_rfc3339(),
    };
    drop(guard);
    state
        .replication
        .lock()
        .expect("lock poisoned")
        .insert(pkg.id.clone(), pkg.clone());
    (StatusCode::CREATED, Json(pkg)).into_response()
}

pub async fn list_public_reports(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_public_reports");
    Json(serde_json::json!({ "reports": state.reports_sorted() }))
}

pub async fn get_public_report(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    info!(report_id = %id, "get_public_report");
    let guard = state.reports.lock().expect("lock poisoned");
    match guard.get(&id) {
        Some(r) => Json(r.clone()).into_response(),
        None => problem(StatusCode::NOT_FOUND, format!("no published report with id {id}")),
    }
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-forensics/forensics.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/forensics/investigations — initiate an investigation.
pub async fn initiate_forensic_investigation(
    State(state): State<SharedState>,
    Json(req): Json<spec::InvestigationRequest>,
) -> impl IntoResponse {
    info!(investigation_type = %req.investigation_type, "initiate_forensic_investigation");
    let now = chrono::Utc::now().to_rfc3339();
    let investigation = spec::Investigation {
        investigation_id: uuid::Uuid::now_v7().to_string(),
        investigation_type: req.investigation_type,
        subject_ref: req.subject_ref,
        trigger_event_id: req.trigger_event_id,
        initiated_by_hash: req.initiated_by_hash,
        status: "initiated".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.investigations.lock().expect("lock poisoned");
    guard.insert(investigation.investigation_id.clone(), investigation.clone());
    drop(guard);
    (StatusCode::CREATED, Json(investigation)).into_response()
}

/// GET /api/v1/governance/forensics/investigations — list investigations.
pub async fn list_forensic_investigations(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    info!("list_forensic_investigations");
    let guard = state.investigations.lock().expect("lock poisoned");
    let mut items: Vec<spec::Investigation> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// GET /api/v1/governance/forensics/investigations/{investigation_id} — fetch one.
pub async fn get_forensic_investigation(
    State(state): State<SharedState>,
    Path(investigation_id): Path<String>,
) -> impl IntoResponse {
    info!(investigation_id, "get_forensic_investigation");
    let guard = state.investigations.lock().expect("lock poisoned");
    match guard.get(&investigation_id) {
        Some(i) => Json(i.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "investigation not found"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/governance/forensics/investigations/{investigation_id}/evidence —
/// attach evidence.
pub async fn attach_evidence(
    State(state): State<SharedState>,
    Path(investigation_id): Path<String>,
    Json(req): Json<spec::EvidenceAttachmentRequest>,
) -> impl IntoResponse {
    info!(investigation_id, "attach_evidence");
    if !state.investigations.lock().expect("lock poisoned").contains_key(&investigation_id) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "investigation not found"})),
        )
            .into_response();
    }
    let attachment = spec::EvidenceAttachment {
        attachment_id: uuid::Uuid::now_v7().to_string(),
        investigation_id: investigation_id.clone(),
        evidence_type: req.evidence_type,
        source: req.source,
        hash: req.hash,
        storage_ref: req.storage_ref,
        attached_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.evidence.lock().expect("lock poisoned");
    guard.entry(investigation_id).or_default().push(attachment.clone());
    drop(guard);
    (StatusCode::CREATED, Json(attachment)).into_response()
}

/// POST /api/v1/governance/forensics/investigations/{investigation_id}/findings —
/// add a finding.
pub async fn add_forensic_finding(
    State(state): State<SharedState>,
    Path(investigation_id): Path<String>,
    Json(req): Json<spec::FindingRequest>,
) -> impl IntoResponse {
    info!(investigation_id, "add_forensic_finding");
    if !state.investigations.lock().expect("lock poisoned").contains_key(&investigation_id) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "investigation not found"})),
        )
            .into_response();
    }
    let finding = spec::Finding {
        finding_id: uuid::Uuid::now_v7().to_string(),
        investigation_id: investigation_id.clone(),
        finding_type: req.finding_type,
        amount_kes: req.amount_kes,
        description: req.description,
        referral: req.referral,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.findings.lock().expect("lock poisoned");
    guard.entry(investigation_id).or_default().push(finding.clone());
    drop(guard);
    (StatusCode::CREATED, Json(finding)).into_response()
}
