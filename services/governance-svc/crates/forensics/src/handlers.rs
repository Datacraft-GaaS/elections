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

pub async fn list_cycles() -> impl IntoResponse {
    info!("list_cycles");
    let cycles: Vec<AnalysisCycle> = vec![];
    Json(serde_json::json!({ "cycles": cycles }))
}

pub async fn create_cycle(Json(req): Json<CreateCycleRequest>) -> impl IntoResponse {
    info!(election_id = %req.election_id, method = ?req.method, "create_cycle");
    let cycle = AnalysisCycle {
        id: uuid::Uuid::new_v4().to_string(),
        election_id: req.election_id,
        method: req.method,
        status: CycleStatus::Draft,
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    (StatusCode::CREATED, Json(cycle))
}

pub async fn trigger_run(Path(id): Path<String>) -> impl IntoResponse {
    info!(cycle_id = %id, "trigger_run");
    StatusCode::ACCEPTED
}

pub async fn list_estimates(Path(id): Path<String>) -> impl IntoResponse {
    info!(cycle_id = %id, "list_estimates");
    let estimates: Vec<EstimateRecord> = vec![];
    Json(serde_json::json!({ "estimates": estimates }))
}

pub async fn create_referral(
    Path(cycle_id): Path<String>,
    Json(req): Json<CreateReferralRequest>,
) -> impl IntoResponse {
    info!(cycle_id, referred_to = ?req.referred_to, "create_referral");
    let packet = ReferralPacket {
        id: uuid::Uuid::new_v4().to_string(),
        cycle_id,
        referred_to: req.referred_to,
        referral_basis: req.referral_basis,
        content_url: req.content_url,
        referred_at: chrono::Utc::now().to_rfc3339(),
        acknowledged_at: None,
    };
    (StatusCode::CREATED, Json(packet))
}

pub async fn publish_replication(
    Path(cycle_id): Path<String>,
    Json(req): Json<PublishReplicationRequest>,
) -> impl IntoResponse {
    info!(cycle_id, "publish_replication");
    let pkg = ReplicationPackage {
        id: uuid::Uuid::new_v4().to_string(),
        cycle_id,
        code_url: req.code_url,
        data_hash: req.data_hash,
        environment_spec: req.environment_spec,
        doi: req.doi,
        published_at: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::CREATED, Json(pkg))
}

pub async fn list_public_reports() -> impl IntoResponse {
    info!("list_public_reports");
    let reports: Vec<PublicReport> = vec![];
    Json(serde_json::json!({ "reports": reports }))
}

pub async fn get_public_report(Path(id): Path<String>) -> impl IntoResponse {
    info!(report_id = %id, "get_public_report");
    Json(PublicReport {
        id,
        ..Default::default()
    })
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
