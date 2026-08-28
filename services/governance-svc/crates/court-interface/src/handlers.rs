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

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn request_evidence(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "request_evidence");
    StatusCode::ACCEPTED
}

pub async fn get_evidence_bundle(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "get_evidence_bundle");
    Json(EvidenceBundle { petition_id, ..Default::default() })
}

pub async fn register_witness(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "register_witness");
    Json(TechnicalWitness { petition_id, ..Default::default() })
}

pub async fn list_witnesses(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "list_witnesses");
    let witnesses: Vec<TechnicalWitness> = vec![];
    Json(serde_json::json!({ "witnesses": witnesses }))
}

pub async fn add_supplementary_request(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "add_supplementary_request");
    Json(SupplementaryRequest { petition_id, ..Default::default() })
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-court/interface.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/court/petitions — file an election petition.
pub async fn file_petition(
    State(state): State<SharedState>,
    Json(req): Json<spec::PetitionRecordRequest>,
) -> impl IntoResponse {
    info!(petition_ref = %req.petition_ref, "file_petition");
    let now = chrono::Utc::now().to_rfc3339();
    let petition = spec::PetitionRecord {
        petition_id: uuid::Uuid::now_v7().to_string(),
        petition_ref: req.petition_ref,
        petitioner_id_hash: req.petitioner_id_hash,
        respondent_code: req.respondent_code,
        contest_code: req.contest_code,
        grounds: req.grounds,
        filed_at: req.filed_at,
        status: "filed".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.petitions.lock().expect("lock poisoned");
    guard.insert(petition.petition_id.clone(), petition.clone());
    drop(guard);
    (StatusCode::CREATED, Json(petition)).into_response()
}

/// GET /api/v1/governance/court/petitions — list election petitions.
pub async fn list_petitions(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_petitions");
    let guard = state.petitions.lock().expect("lock poisoned");
    let mut items: Vec<spec::PetitionRecord> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.filed_at.cmp(&b.filed_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// GET /api/v1/governance/court/petitions/{petition_id} — fetch one petition.
pub async fn get_petition(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
) -> impl IntoResponse {
    info!(petition_id, "get_petition");
    let guard = state.petitions.lock().expect("lock poisoned");
    match guard.get(&petition_id) {
        Some(p) => Json(p.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "petition not found"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/governance/court/petitions/{petition_id}/evidence-requests —
/// create an evidence request.
pub async fn create_evidence_request(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
    Json(req): Json<spec::EvidenceRequestInput>,
) -> impl IntoResponse {
    info!(petition_id, "create_evidence_request");
    if !state.petitions.lock().expect("lock poisoned").contains_key(&petition_id) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "petition not found"})),
        )
            .into_response();
    }
    let evidence = spec::EvidenceRequest {
        request_id: uuid::Uuid::now_v7().to_string(),
        evidence_types: req.evidence_types,
        deadline: req.deadline,
        court_ref: req.court_ref,
        package_url: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.evidence_requests.lock().expect("lock poisoned");
    guard.entry(petition_id).or_default().push(evidence.clone());
    drop(guard);
    (StatusCode::CREATED, Json(evidence)).into_response()
}

/// POST /api/v1/governance/court/petitions/{petition_id}/judgments — record a
/// judgment and advance the petition status.
pub async fn record_judgment(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
    Json(req): Json<spec::CourtJudgment>,
) -> impl IntoResponse {
    info!(petition_id, outcome = %req.outcome, "record_judgment");
    let mut guard = state.petitions.lock().expect("lock poisoned");
    match guard.get_mut(&petition_id) {
        Some(p) => {
            p.status = "judgment".to_string();
            p.updated_at = chrono::Utc::now().to_rfc3339();
            Json(req).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "petition not found"})),
        )
            .into_response(),
    }
}
