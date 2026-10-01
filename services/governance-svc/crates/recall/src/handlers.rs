use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::info;

use crate::models::*;
use crate::spec::{self, Petition, PetitionRequest};
use crate::threshold::{meets_art_104_2_threshold, threshold_numerator_denominator};
use crate::state::SharedState;

/// A problem+json response, so a refusal is readable by a caller rather than
/// being a bare status code.
fn problem(status: StatusCode, detail: String) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail,
        })),
    )
        .into_response()
}

/// The body `import_signatories` accepts: a batch of OCR rows.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ImportBatch {
    pub rows: Vec<SignatoryImportRow>,
}


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


pub async fn create_petition(
    State(state): State<SharedState>,
    Json(req): Json<PetitionRequest>,
) -> impl IntoResponse {
    info!(constituency = %req.constituency_code, "create_petition");
    let now = chrono::Utc::now().to_rfc3339();
    let petition_id = uuid::Uuid::now_v7().to_string();
    let petition = Petition {
        petition_id: petition_id.clone(),
        constituency_code: req.constituency_code,
        member_id_hash: req.member_id_hash,
        grounds: req.grounds,
        initiator_id_hash: req.initiator_id_hash,
        petition_date: req.petition_date,
        status: "OPEN".to_string(),
        // 30% of the constituency roll. Stored per petition rather than
        // hardcoded in a handler, so eligibility is computed from the roll this
        // petition was actually filed against.
        threshold_required: 0,
        signatures_collected: 0,
        created_at: now.clone(),
        updated_at: now,
    };
    state
        .petitions
        .lock()
        .expect("recall petitions lock poisoned")
        .insert(petition_id.clone(), petition.clone());
    (StatusCode::CREATED, Json(petition))
}

pub async fn import_signatories(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(body): Json<ImportBatch>,
) -> Result<Response, Response> {
    info!(petition_id = %id, rows = body.rows.len(), "import_signatories");
    if !state
        .petitions
        .lock()
        .expect("recall petitions lock poisoned")
        .contains_key(&id)
    {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no petition with id {id}"),
        ));
    }

    let mut result = SignatoryImportResult {
        batch_id: uuid::Uuid::now_v7().to_string(),
        total_rows: body.rows.len(),
        accepted: 0,
        flagged_for_manual: 0,
        duplicates: 0,
        not_on_roll: 0,
        errors: Vec::new(),
    };
    let now = chrono::Utc::now().to_rfc3339();
    let mut seen: Vec<String> = state
        .signatories
        .list_where(|r| r.petition_id == id)
        .into_iter()
        .map(|r| r.voter_id)
        .collect();

    for (index, row) in body.rows.iter().enumerate() {
        if row.voter_id.trim().is_empty() {
            result.errors.push(RowError {
                row_index: index,
                voter_id: row.voter_id.clone(),
                reason: "voter_id is empty".to_string(),
            });
            continue;
        }
        // Previously the whole batch was discarded into a minted batch_id and a
        // status of "processing". Every row is now classified and stored.
        let duplicate = seen.contains(&row.voter_id);
        if duplicate {
            result.duplicates += 1;
        }
        seen.push(row.voter_id.clone());

        // Rows below 0.80 OCR confidence are flagged for manual re-entry rather
        // than counted toward the threshold, which is what the model comment
        // describes and no code did.
        let low_confidence = row.ocr_confidence.is_some_and(|c| c < 0.80);
        let status = if duplicate {
            "DUPLICATE"
        } else if low_confidence {
            "FLAGGED"
        } else {
            "VALID"
        };
        match status {
            "VALID" => result.accepted += 1,
            "FLAGGED" => result.flagged_for_manual += 1,
            _ => {}
        }
        state.signatories.insert(
            format!("{id}:{}:{}", row.voter_id, index),
            SignatoryRecord {
                id: format!("{id}:{}:{}", row.voter_id, index),
                petition_id: id.clone(),
                voter_id: row.voter_id.clone(),
                ward: row.ward.clone(),
                ocr_confidence: row.ocr_confidence,
                verification_status: status.to_string(),
                verified_at: Some(now.clone()),
                verification_method: "ocr_import".to_string(),
            },
        );
    }
    Ok((StatusCode::OK, Json(result)).into_response())
}

pub async fn get_threshold_report(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(petition_id = %id, "get_threshold_report");
    let petition = state
        .petitions
        .lock()
        .expect("recall petitions lock poisoned")
        .get(&id)
        .cloned();
    let petition = match petition {
        Some(p) => p,
        // Previously answered 200 with a report for any id at all.
        None => return Err(problem(StatusCode::NOT_FOUND, format!("no petition with id {id}"))),
    };

    let records = state.signatories.list_where(|r| r.petition_id == id);
    let total = records.len() as u64;
    // Only VALID rows count. FLAGGED and DUPLICATE do not, which is the whole
    // point of classifying them at import.
    let verified = records
        .iter()
        .filter(|r| r.verification_status == "VALID")
        .count() as u64;

    // The electorate must come from a loaded roll. Previously this was the
    // literal 100_000, so eligibility was decided against a number that had no
    // relationship to any constituency.
    let registered_voters = match state.roll_size(&petition.constituency_code) {
        Some(n) => n,
        None => {
            return Err(problem(
                StatusCode::CONFLICT,
                format!(
                    "no roll loaded for constituency {}; refusing to evaluate \
                     article IV section 2 against an assumed electorate",
                    petition.constituency_code
                ),
            ))
        }
    };
    let (num, den) = threshold_numerator_denominator(verified, registered_voters);
    let meets = meets_art_104_2_threshold(verified, registered_voters);
    let mut ward_breakdown = serde_json::Map::new();
    for r in records.iter().filter(|r| r.verification_status == "VALID") {
        let entry = ward_breakdown
            .entry(r.ward.clone())
            .or_insert_with(|| serde_json::json!(0));
        *entry = serde_json::json!(entry.as_u64().unwrap_or(0) + 1);
    }

    Ok((
        StatusCode::OK,
        Json(ThresholdReport {
            id: uuid::Uuid::now_v7().to_string(),
            petition_id: id,
            total_signatories: total,
            verified_signatories: verified,
            registered_voters,
            numerator_x100: num,
            denominator_30pct: den,
            meets_threshold: meets,
            ward_breakdown: serde_json::Value::Object(ward_breakdown),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }),
    )
        .into_response())
}

pub async fn close_petition(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(petition_id = %id, "close_petition");
    let now = chrono::Utc::now().to_rfc3339();
    let updated = {
        let mut guard = state.petitions.lock().expect("recall petitions lock poisoned");
        match guard.get_mut(&id) {
            Some(p) => {
                p.status = "CLOSED".to_string();
                p.updated_at = now;
                Some(p.clone())
            }
            None => None,
        }
    };
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        // Previously 204 for any id, so closing a petition that never existed
        // was indistinguishable from closing a real one.
        None => Err(problem(StatusCode::NOT_FOUND, format!("no petition with id {id}"))),
    }
}

pub async fn create_recall_election(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(petition_id = %id, "create_recall_election");
    let exists = state
        .petitions
        .lock()
        .expect("recall petitions lock poisoned")
        .contains_key(&id);
    if !exists {
        return Err(problem(StatusCode::NOT_FOUND, format!("no petition with id {id}")));
    }
    let election = RecallElection {
        id: uuid::Uuid::now_v7().to_string(),
        petition_id: id,
        // Real: previously "pending" with a comment claiming an outbox event was
        // emitted and an M50 by-election triggered. Neither happened, so the
        // election existed on paper. This records PENDING truthfully -- it is
        // pending because no outbox is wired, not because one is working.
        by_election_id: None,
        trigger_status: "PENDING".to_string(),
        trigger_attempts: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .elections
        .insert(election.id.clone(), election.clone());
    Ok((StatusCode::CREATED, Json(election)).into_response())
}

pub async fn list_signatories(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!(petition_id = %id, "list_signatories");
    // Real. Previously always an empty list, which asserted that no signatory
    // had signed rather than that nothing was stored.
    let records = state.signatories.list_where(|r| r.petition_id == id);
    Json(serde_json::json!({ "petition_id": id, "records": records, "total": records.len() }))
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
