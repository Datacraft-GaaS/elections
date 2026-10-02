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
    response::{IntoResponse, Response},
    Json,
};
use tracing::info;

use crate::models::{EvidenceBundle, EvidenceItem, SupplementaryRequest, TechnicalWitness};
use crate::spec::{self, EvidenceRequest};
use crate::state::SharedState;

/// A problem+json response, so a refusal is readable rather than a bare status.
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

/// 404 unless the petition was actually filed.
///
/// Every handler in this group keys off a petition id. Returning a
/// default-constructed record for an id that was never filed is what made
/// `get_evidence_bundle` answer 200 with an empty bundle for any petition.
fn require_petition(state: &SharedState, petition_id: &str) -> Result<(), Response> {
    if state.has_petition(petition_id) {
        Ok(())
    } else {
        Err(problem(
            StatusCode::NOT_FOUND,
            format!("no petition with id {petition_id}"),
        ))
    }
}

/// The body `register_witness` accepts.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WitnessInput {
    pub full_name: String,
    pub expertise: String,
    pub organisation: String,
}

/// The body `add_supplementary_request` accepts.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SupplementaryInput {
    pub description: String,
    pub requested_by: String,
}


pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}


pub async fn request_evidence(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
    Json(req): Json<spec::EvidenceRequestInput>,
) -> Result<Response, Response> {
    info!(petition_id, "request_evidence");
    require_petition(&state, &petition_id)?;
    let request = EvidenceRequest {
        request_id: uuid::Uuid::now_v7().to_string(),
        evidence_types: req.evidence_types,
        deadline: req.deadline,
        court_ref: req.court_ref,
        package_url: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    // Real. Previously answered 202 for a request that was never made; the only
    // trace of it was a log line.
    state
        .evidence_requests
        .lock()
        .expect("court evidence requests lock poisoned")
        .entry(petition_id)
        .or_default()
        .push(request.clone());
    Ok((StatusCode::ACCEPTED, Json(request)).into_response())
}

pub async fn get_evidence_bundle(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
) -> Result<Response, Response> {
    info!(petition_id, "get_evidence_bundle");
    require_petition(&state, &petition_id)?;
    let requests = state.requests_for(&petition_id);
    if requests.is_empty() {
        // No request has been made, so no bundle exists. This used to answer 200
        // with a well-formed default bundle containing nothing, and from the
        // response alone an empty bundle and a populated one are identical --
        // which in a filing context is worse than an error.
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no evidence requested for petition {petition_id}"),
        ));
    }
    // The bundle is assembled from the requests actually recorded against the
    // petition, so it is a statement about what was asked for.
    let items: Vec<EvidenceItem> = requests
        .iter()
        .map(|r| EvidenceItem {
            item_id: r.request_id.clone(),
            item_type: r.evidence_types.join(","),
            description: format!("evidence request ({})", r.court_ref),
            document_ref: r.package_url.clone().unwrap_or_default(),
            hash: String::new(),
            submitted_at: r.created_at.clone(),
            submitted_by: r.court_ref.clone(),
            verified: false,
            verification_note: "request recorded; evidence not yet lodged".to_string(),
            source_system: "court-interface".to_string(),
            retention_until: r.deadline.clone(),
            classification: "UNCLASSIFIED".to_string(),
            chain_of_custody: "not_lodged".to_string(),
            exhibit_ref: String::new(),
            court_ref: r.court_ref.clone(),
        })
        .collect();
    let bundle = EvidenceBundle {
        bundle_id: uuid::Uuid::now_v7().to_string(),
        petition_id,
        items,
    };
    Ok(Json(bundle).into_response())
}

pub async fn register_witness(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
    Json(body): Json<WitnessInput>,
) -> Result<Response, Response> {
    info!(petition_id, "register_witness");
    require_petition(&state, &petition_id)?;
    let witness = TechnicalWitness {
        witness_id: uuid::Uuid::now_v7().to_string(),
        petition_id: petition_id.clone(),
        full_name: body.full_name,
        expertise: body.expertise,
        organisation: body.organisation,
        registered_at: chrono::Utc::now().to_rfc3339(),
    };
    // Real. Previously answered 200 with a default-constructed TechnicalWitness,
    // so a witness appeared registered to the petition having never consented,
    // been identified, or been contactable.
    state
        .witnesses
        .insert(witness.witness_id.clone(), witness.clone());
    Ok((StatusCode::CREATED, Json(witness)).into_response())
}

pub async fn list_witnesses(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
) -> Result<Response, Response> {
    info!(petition_id, "list_witnesses");
    require_petition(&state, &petition_id)?;
    // Real. Previously always an empty array, which asserted that no witness was
    // registered to the petition rather than that nothing was stored.
    let witnesses = state
        .witnesses
        .list_where(|w| w.petition_id == petition_id);
    Ok(Json(serde_json::json!({ "witnesses": witnesses })).into_response())
}

pub async fn add_supplementary_request(
    State(state): State<SharedState>,
    Path(petition_id): Path<String>,
    Json(body): Json<SupplementaryInput>,
) -> Result<Response, Response> {
    info!(petition_id, "add_supplementary_request");
    require_petition(&state, &petition_id)?;
    let request = SupplementaryRequest {
        request_id: uuid::Uuid::now_v7().to_string(),
        petition_id: petition_id.clone(),
        description: body.description,
        requested_by: body.requested_by,
        requested_at: chrono::Utc::now().to_rfc3339(),
        status: "OPEN".to_string(),
    };
    // Real. Previously echoed the petition id into a default-constructed
    // SupplementaryRequest for the petition.
    state
        .supplementary
        .insert(request.request_id.clone(), request.clone());
    Ok((StatusCode::CREATED, Json(request)).into_response())
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
