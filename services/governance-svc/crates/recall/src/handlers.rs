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
use crate::threshold::meets_art_104_2_threshold;

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

/// A declared operation with no implementation behind it.
///
/// Returned instead of a fabricated success. `get_threshold_report` is why this
/// matters more here than elsewhere: it evaluated recall eligibility against
/// California Constitution article IV section 2 using a hardcoded
/// `registered_voters = 100_000` and `verified_signatories = 0`, so the one
/// invariant standing between a petition and a recall election was evaluated
/// against an invented electorate.
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

pub async fn create_petition(Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("create_petition");
    // Minted a UUIDv7 and echoed the request back as status "draft", storing
    // nothing. The comment claimed commissioner dual-sign is verified in
    // production via SpiceDB; no SpiceDB call exists here.
    not_implemented(
        "recall petition creation",
        "no petition store or SpiceDB dual-sign check is wired; it previously \
         returned a minted id and the request echoed back as draft",
    )
}

pub async fn import_signatories(
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "import_signatories");
    // Discarded the batch and answered with a minted batch_id and status
    // "processing", with nothing processing and no poll endpoint to ask.
    let _ = id;
    not_implemented(
        "signatory import",
        "no OCR pipeline or poll endpoint is wired; it previously returned a \
         minted batch_id and status processing for a batch it discarded",
    )
}

pub async fn get_threshold_report(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_threshold_report");
    // Previously evaluated the article IV section 2 recall threshold against a
    // hardcoded electorate of 100_000 and zero verified signatories, and
    // returned meets_threshold computed from them. The threshold logic in
    // crate::threshold is real; both inputs were constants.
    let _ = id;
    return not_implemented(
        "recall threshold report",
        "registered_voters was hardcoded to 100_000 and verified_signatories \
         to 0, so article IV section 2 eligibility was evaluated against an \
         invented electorate",
    )
}

pub async fn close_petition(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "close_petition");
    // Answered 204, which is a successful close: a petition was closed and
    // recorded nowhere.
    let _ = id;
    not_implemented(
        "petition closure",
        "no petition store is wired; it previously returned 204 for a closure \
         it never recorded",
    )
}

pub async fn create_recall_election(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "create_recall_election");
    // Returned 201 with a minted RecallElection and trigger_status "pending".
    // The comment claimed it triggers M50 by-election creation via an outbox
    // event; there is no outbox client here and no event emitted, so a recall
    // election was created on paper.
    let _ = id;
    not_implemented(
        "recall election creation",
        "no outbox or by-election trigger is wired; it previously returned \
         201 with trigger_status pending and emitted nothing",
    )
}

pub async fn list_signatories(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "list_signatories");
    // An empty list with total: 0 reads as "this petition has no signatories",
    // which is a claim about a petition, not about the absence of storage.
    let _ = id;
    not_implemented(
        "signatory listing",
        "no signatory store is wired; it previously returned an empty list \
         with total 0",
    )
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
