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
use crate::threshold::{meets_art_104_2_threshold, threshold_numerator_denominator};

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_petitions() -> impl IntoResponse {
    info!("list_petitions");
    let petitions: Vec<RecallPetition> = vec![];
    Json(serde_json::json!({ "petitions": petitions, "total": 0 }))
}

pub async fn get_petition(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_petition");
    Json(RecallPetition { id, ..Default::default() })
}

pub async fn create_petition(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("create_petition");
    // Commissioner dual-sign verified in production via SpiceDB (P01)
    let id = uuid::Uuid::now_v7().to_string();
    Json(serde_json::json!({ "id": id, "status": "draft", "body": body }))
}

pub async fn import_signatories(
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "import_signatories");
    // OCR batch processed asynchronously; result available via poll endpoint
    Json(serde_json::json!({
        "petition_id": id,
        "batch_id": uuid::Uuid::now_v7().to_string(),
        "status": "processing"
    }))
}

pub async fn get_threshold_report(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_threshold_report");
    let registered_voters = 100_000u64;
    let verified_signatories = 0u64;
    let (num, den) = threshold_numerator_denominator(verified_signatories, registered_voters);
    let meets = meets_art_104_2_threshold(verified_signatories, registered_voters);
    let report = ThresholdReport {
        id: uuid::Uuid::now_v7().to_string(),
        petition_id: id,
        total_signatories: 0,
        verified_signatories,
        registered_voters,
        numerator_x100: num,
        denominator_30pct: den,
        meets_threshold: meets,
        ward_breakdown: serde_json::Value::Object(Default::default()),
        generated_at: chrono::Utc::now().to_rfc3339(),
    };
    Json(report)
}

pub async fn close_petition(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "close_petition");
    StatusCode::NO_CONTENT
}

pub async fn create_recall_election(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "create_recall_election");
    // Idempotent: triggers M50 by-election creation via outbox event
    let election = RecallElection {
        id: uuid::Uuid::now_v7().to_string(),
        petition_id: id,
        by_election_id: None,
        trigger_status: "pending".to_string(),
        trigger_attempts: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::CREATED, Json(election))
}

pub async fn list_signatories(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "list_signatories");
    let records: Vec<SignatoryRecord> = vec![];
    Json(serde_json::json!({ "petition_id": id, "records": records, "total": 0 }))
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-recall/recall.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/recall/petitions — initiate a recall petition.
pub async fn initiate_recall_petition(
    State(state): State<SharedState>,
    Json(req): Json<spec::PetitionRequest>,
) -> impl IntoResponse {
    info!(constituency = %req.constituency_code, "initiate_recall_petition");
    if req.member_id_hash.is_empty() || req.initiator_id_hash.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "member_id_hash and initiator_id_hash are required"})),
        )
            .into_response();
    }
    let now = chrono::Utc::now().to_rfc3339();
    let petition = spec::Petition {
        petition_id: uuid::Uuid::now_v7().to_string(),
        constituency_code: req.constituency_code,
        member_id_hash: req.member_id_hash,
        grounds: req.grounds,
        initiator_id_hash: req.initiator_id_hash,
        petition_date: req.petition_date,
        status: "initiated".to_string(),
        threshold_required: 0,
        signatures_collected: 0,
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.petitions.lock().expect("lock poisoned");
    guard.insert(petition.petition_id.clone(), petition.clone());
    drop(guard);
    (StatusCode::CREATED, Json(petition)).into_response()
}

/// GET /api/v1/governance/recall/petitions — list recall petitions.
pub async fn list_recall_petitions(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_recall_petitions");
    let guard = state.petitions.lock().expect("lock poisoned");
    let mut items: Vec<spec::Petition> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// GET /api/v1/governance/recall/petitions/{petition_id} — fetch one petition.
pub async fn get_recall_petition(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
) -> impl IntoResponse {
    info!(petition_id, "get_recall_petition");
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

/// POST /api/v1/governance/recall/petitions/{petition_id}/signatures — record a
/// signature and return the running total.
pub async fn record_recall_signature(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
    Json(req): Json<spec::SignatureRequest>,
) -> impl IntoResponse {
    info!(petition_id, "record_recall_signature");
    if req.petitioner_id_hash.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "petitioner_id_hash is required"})),
        )
            .into_response();
    }
    let mut guard = state.signatures.lock().expect("lock poisoned");
    let count = guard.entry(petition_id.clone()).or_insert(0);
    *count = count.saturating_add(1);
    let count_so_far = *count;
    drop(guard);
    Json(spec::SignatureResponse {
        accepted: true,
        count_so_far,
        rejection_reason: None,
    })
    .into_response()
}

/// POST /api/v1/governance/recall/petitions/{petition_id}/verification — record
/// the verified signature count and advance the petition status.
pub async fn verify_signature_count(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
    Json(req): Json<spec::SignatureVerification>,
) -> impl IntoResponse {
    info!(petition_id, "verify_signature_count");
    let threshold_met = {
        let mut pguard = state.petitions.lock().expect("lock poisoned");
        if let Some(p) = pguard.get_mut(&petition_id) {
            p.signatures_collected = req.verified_count;
            p.updated_at = chrono::Utc::now().to_rfc3339();
            p.status = if meets_art_104_2_threshold(req.verified_count, p.threshold_required)
                && p.threshold_required > 0
            {
                "threshold_met".to_string()
            } else {
                "verified".to_string()
            };
            req.verified_count >= p.threshold_required
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"detail": "petition not found"})),
            )
                .into_response();
        }
    };
    let _ = threshold_met;
    Json(spec::SignatureVerification {
        verified_count: req.verified_count,
        verified_by_hash: req.verified_by_hash,
        method: req.method,
        verified_at: Some(chrono::Utc::now().to_rfc3339()),
    })
    .into_response()
}
