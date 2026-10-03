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

//! Axum sub-router for the declaration crate (WP-3.3, M51).
//! Merged into rts-api's main Router at boot via Router::merge().
//!
//! Path prefix: /api/v1/declarations
//!
//! Every handler here reads or advances the real [`crate::aggregate::Declaration`]
//! aggregate through [`crate::store::AppState`]. Previously they all answered from
//! the path id alone: `get_declaration` reported `status: "pending"` for any id,
//! `declare_result` reported `status: "declared"` for any id with no winner and no
//! evaluation, and `set_runoff` / `set_tie` asserted a run-off or tie against a
//! declaration that need not exist. The Art. 138(4) rules engine in
//! [`crate::rules`] was never called from anywhere.
//!
//! The rules engine is the authority on the outcome. A handler that reports
//! `declared` is reporting what `rules::evaluate` returned for the tally that was
//! supplied, not what the caller asked for.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::aggregate::DeclarationStatus;
use crate::events::*;
use crate::rules::{evaluate, DeclarationOutcome, PresidentialTally};
use crate::store::{may_issue_certificate, window_status_wire, SharedState};

// ---- request/response models ------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct InitiateDeclarationRequest {
    pub contest_id: String,
    pub election_id: String,
    pub tally_snapshot_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeclarationCreated {
    pub declaration_id: String,
    pub status: String,
}

/// A declaration submission.
///
/// Carries the tally the declaration is made from, because the outcome is derived
/// from it. Without the tally there is nothing to evaluate, and a "declared"
/// status with no tally is exactly the fabrication this crate must not produce.
#[derive(Debug, Deserialize)]
pub struct DeclareRequest {
    pub declared_by: String,
    pub ceremony_id: String,
    pub hsm_signature: String,
    /// The signed tally snapshot the declaration is made from.
    pub tally: PresidentialTally,
}

#[derive(Debug, Deserialize)]
pub struct AmendRequest {
    pub reason: String,
    pub amended_by: String,
    pub approved_by: String,
    pub previous_snapshot_hash: String,
    pub new_snapshot_hash: String,
    pub court_order_ref: Option<String>,
    pub new_tally_event_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JudiciaryAckRequest {
    pub registrar_id: String,
    pub acknowledged_at: String,
}

#[derive(Debug, Serialize)]
pub struct DeclarationState {
    pub declaration_id: String,
    pub status: String,
    pub winner_id: Option<String>,
    pub declared_at: Option<String>,
    /// How the status was reached.
    pub outcome: Option<DeclarationOutcome>,
    /// Why the declaration cannot be completed, when it cannot.
    pub blocking_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PetitionWindowState {
    pub declaration_id: String,
    pub status: String,
    pub window_open_at: Option<String>,
    pub window_close_at: Option<String>,
    pub petition_court_case: Option<String>,
    pub judiciary_ack_at: Option<String>,
}

// ---- helpers ----------------------------------------------------------------

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

/// The status a declaration is in, as it appears on the wire.
fn status_wire(status: &DeclarationStatus) -> &'static str {
    match status {
        DeclarationStatus::Pending => "pending",
        DeclarationStatus::Declared => "declared",
        DeclarationStatus::RunoffRequired => "runoff_required",
        DeclarationStatus::TiePending => "tie_pending",
        DeclarationStatus::Amended => "amended",
        DeclarationStatus::Annulled => "annulled",
    }
}

/// The status a declaration is in, as the aggregate reports it.
fn status_json(status: &DeclarationStatus) -> String {
    status_wire(status).to_string()
}

/// The event that records a rules-engine outcome.
fn outcome_event(outcome: &DeclarationOutcome) -> (&'static str, serde_json::Value) {
    match outcome {
        DeclarationOutcome::Declared { winner_id } => (
            EVENT_RESULT_DECLARED,
            serde_json::json!({ "winner_id": winner_id }),
        ),
        DeclarationOutcome::RunoffRequired {
            runoff_candidates,
            counties_above_threshold_leader,
        } => (
            EVENT_RUNOFF_REQUIRED,
            serde_json::json!({
                "runoff_candidates": runoff_candidates,
                "counties_above_threshold_leader": counties_above_threshold_leader,
            }),
        ),
        DeclarationOutcome::TiePending { tied_candidates } => (
            EVENT_TIE_PENDING,
            serde_json::json!({ "tied_candidates": tied_candidates }),
        ),
    }
}

// ---- handlers ---------------------------------------------------------------

async fn initiate_declaration(
    State(state): State<SharedState>,
    Json(req): Json<InitiateDeclarationRequest>,
) -> axum::response::Response {
    if req.contest_id.trim().is_empty() || req.election_id.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "contest_id and election_id are required",
        );
    }
    let id = state.initiate(&req.contest_id, &req.election_id, &req.tally_snapshot_id);
    info!(contest_id = %req.contest_id, declaration_id = %id, "declaration: initiate");
    (
        StatusCode::CREATED,
        Json(DeclarationCreated {
            declaration_id: id,
            status: "pending".to_string(),
        }),
    )
        .into_response()
}

async fn declare_result(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<DeclareRequest>,
) -> axum::response::Response {
    info!(
        declaration_id = %id,
        declared_by = %req.declared_by,
        ceremony_id = %req.ceremony_id,
        "declaration: declare"
    );
    let Some(current) = state.get(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    // A declaration is made once. Re-declaring would silently replace the
    // outcome the first declaration produced.
    if current.status != DeclarationStatus::Pending {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "declaration is {}; only a pending declaration can be made",
                status_wire(&current.status)
            ),
        );
    }

    // The rules engine decides. Elections Act Art. 138(4) and §§83-84: a winner
    // needs BOTH a strict majority and the 25%-in-24-of-47-counties threshold.
    let outcome = evaluate(&req.tally);
    let (event_type, mut payload) = outcome_event(&outcome);
    payload["contest_id"] = serde_json::json!(current.contest_id);
    payload["election_id"] = serde_json::json!(current.election_id);
    payload["declared_by"] = serde_json::json!(req.declared_by);
    payload["ceremony_id"] = serde_json::json!(req.ceremony_id);
    payload["declared_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    payload["tally_snapshot_id"] = serde_json::json!(current.tally_snapshot_id);
    // The ceremony signature is recorded as supplied. It is NOT verified here:
    // verification needs the ceremony's public key, which lives in the HSM
    // signer service, not in this process. See the blocking reason below.
    payload["hsm_signature"] = serde_json::json!(req.hsm_signature);

    if let Err(e) = state.apply(&id, event_type, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }

    let declaration = match state.get(&id) {
        Some(d) => d,
        None => {
            return problem(
                StatusCode::NOT_FOUND,
                format!("no declaration with id {id}"),
            )
        }
    };
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "status": status_json(&declaration.status),
            "winner_id": declaration.winner_id,
            "declared_at": declaration.declared_at,
            "ceremony_id": declaration.ceremony_id,
            "outcome": outcome,
            "blocking_reason": "the ceremony signature is recorded as supplied but not \
                                verified here; verification needs the HSM signer's public key, \
                                which is not reachable from this process",
        })),
    )
        .into_response()
}

/// Assert a run-off, but only where the rules engine actually produced one.
async fn set_runoff(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(tally): Json<PresidentialTally>,
) -> axum::response::Response {
    info!(declaration_id = %id, "declaration: runoff");
    let Some(current) = state.get(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    let outcome = evaluate(&tally);
    if !matches!(outcome, DeclarationOutcome::RunoffRequired { .. }) {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "this tally produces {:?}, not a run-off; the outcome is derived from the \
                 tally, not asserted",
                match &outcome {
                    DeclarationOutcome::Declared { winner_id } => format!("declared ({winner_id})"),
                    DeclarationOutcome::TiePending { .. } => "tie_pending".to_string(),
                    DeclarationOutcome::RunoffRequired { .. } => "runoff_required".to_string(),
                }
            ),
        );
    }
    let (event_type, payload) = outcome_event(&outcome);
    if let Err(e) = state.apply(&id, event_type, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    let declaration = state.get(&id).unwrap_or(current);
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "status": status_json(&declaration.status),
            "outcome": outcome,
        })),
    )
        .into_response()
}

/// Assert a tie, but only where the rules engine actually produced one.
async fn set_tie(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(tally): Json<PresidentialTally>,
) -> axum::response::Response {
    info!(declaration_id = %id, "declaration: tie_pending");
    let Some(current) = state.get(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    let outcome = evaluate(&tally);
    if !matches!(outcome, DeclarationOutcome::TiePending { .. }) {
        return problem(
            StatusCode::CONFLICT,
            "this tally does not produce a tie; the outcome is derived from the tally, not asserted",
        );
    }
    let (event_type, payload) = outcome_event(&outcome);
    if let Err(e) = state.apply(&id, event_type, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    let declaration = state.get(&id).unwrap_or(current);
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "status": status_json(&declaration.status),
            "outcome": outcome,
        })),
    )
        .into_response()
}

async fn amend_declaration(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AmendRequest>,
) -> axum::response::Response {
    info!(
        declaration_id = %id,
        amended_by = %req.amended_by,
        "declaration: amend"
    );
    let Some(current) = state.get(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    // An amendment re-declares a result under a court order. A pending
    // declaration has no result to amend.
    if current.status == DeclarationStatus::Pending {
        return problem(
            StatusCode::CONFLICT,
            "a pending declaration has no result to amend",
        );
    }
    // The caller must name the snapshot it is amending. Without that check an
    // amendment could be applied on top of any tally, which is the whole
    // failure mode the lineage field exists to prevent.
    if let Some(existing) = &current.tally_snapshot_id {
        if !existing.is_empty() && existing != &req.previous_snapshot_hash {
            return problem(
                StatusCode::CONFLICT,
                format!(
                    "previous_snapshot_hash is {} but the declaration is on {existing}; \
                     an amendment must name the tally it amends",
                    req.previous_snapshot_hash
                ),
            );
        }
    }
    if req.new_snapshot_hash.trim().is_empty() || req.reason.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "reason and new_snapshot_hash are required",
        );
    }
    let amendment_id = Uuid::now_v7().to_string();
    let payload = serde_json::json!({
        "amendment_id": amendment_id,
        "reason": req.reason,
        "amended_by": req.amended_by,
        "approved_by": req.approved_by,
        "previous_snapshot_hash": req.previous_snapshot_hash,
        "new_snapshot_hash": req.new_snapshot_hash,
        "court_order_ref": req.court_order_ref,
        "new_tally_event_id": req.new_tally_event_id,
    });
    if let Err(e) = state.apply(&id, EVENT_RESULT_AMENDED, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    let declaration = match state.get(&id) {
        Some(d) => d,
        None => {
            return problem(
                StatusCode::NOT_FOUND,
                format!("no declaration with id {id}"),
            )
        }
    };
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "amendment_id": amendment_id,
            "status": status_json(&declaration.status),
            // The lineage is append-only and never shrinks, so a reader can see
            // how many times a result has been amended.
            "amendment_count": declaration.amendment_count(),
            "tally_snapshot_id": declaration.tally_snapshot_id,
        })),
    )
        .into_response()
}

async fn get_declaration(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let Some(declaration) = state.get(&id) else {
        // Previously any id produced `status: "pending"`, which reads as a real
        // declaration that simply had not been made.
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    let outcome = match &declaration.status {
        DeclarationStatus::RunoffRequired => Some(DeclarationOutcome::RunoffRequired {
            runoff_candidates: declaration.runoff_candidates.clone(),
            counties_above_threshold_leader: 0,
        }),
        DeclarationStatus::TiePending => Some(DeclarationOutcome::TiePending {
            tied_candidates: declaration.tied_candidates.clone(),
        }),
        _ => None,
    };
    Json(DeclarationState {
        declaration_id: id,
        status: status_json(&declaration.status),
        winner_id: declaration.winner_id.clone(),
        declared_at: declaration.declared_at.clone(),
        outcome,
        blocking_reason: Some(
            "the ceremony signature is recorded as supplied but not verified here; \
             verification needs the HSM signer's public key"
                .to_string(),
        ),
    })
    .into_response()
}

async fn get_certificate(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let Some(declaration) = state.get(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    // EC-LEG-06: a certificate must never be issued while a declaration is
    // tie-pending. The aggregate owns that rule; it used to be unreachable
    // because this handler answered `not_issued` without reading anything.
    if !may_issue_certificate(&declaration) {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "declaration is {}; a certificate requires a declared result, and a \
                 tie-pending declaration may never be certified",
                status_wire(&declaration.status)
            ),
        );
    }
    Json(serde_json::json!({
        "declaration_id": id,
        "status": "issued",
        "winner_id": declaration.winner_id,
        "declared_at": declaration.declared_at,
    }))
    .into_response()
}

async fn get_petition_window(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let Some(tracker) = state.window(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    Json(PetitionWindowState {
        declaration_id: id,
        status: window_status_wire(&tracker.status).to_string(),
        window_open_at: tracker.window_open_at.clone(),
        window_close_at: tracker.window_close_at.clone(),
        petition_court_case: tracker.petition_court_case.clone(),
        judiciary_ack_at: tracker.judiciary_ack_at.clone(),
    })
    .into_response()
}

async fn judiciary_ack(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<JudiciaryAckRequest>,
) -> axum::response::Response {
    info!(
        declaration_id = %id,
        registrar_id = %req.registrar_id,
        "declaration: judiciary_ack"
    );
    let Some(tracker) = state.window(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no declaration with id {id}"),
        );
    };
    // An acknowledgement follows a petition. Acknowledging a window that has
    // never been petitioned records a judicial act against nothing.
    if tracker.status != crate::aggregate::PetitionWindowStatus::Petitioned {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "petition window is {}; an acknowledgement follows a petition",
                window_status_wire(&tracker.status)
            ),
        );
    }
    let payload = serde_json::json!({
        "acknowledged_at": req.acknowledged_at,
        "registrar_id": req.registrar_id,
    });
    if let Err(e) = state.apply_window(&id, EVENT_JUDICIARY_ACK, payload) {
        return problem(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "acknowledged_at": req.acknowledged_at,
            "registrar_id": req.registrar_id,
        })),
    )
        .into_response()
}

// ---- router -----------------------------------------------------------------

/// Build the sub-router.  Call `Router::merge(declaration::api::router(state))`
/// from rts-api's build_router() to mount these paths.
pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/declarations", post(initiate_declaration))
        .route("/api/v1/declarations/{id}", get(get_declaration))
        .route("/api/v1/declarations/{id}/declare", put(declare_result))
        .route("/api/v1/declarations/{id}/runoff", put(set_runoff))
        .route("/api/v1/declarations/{id}/tie", put(set_tie))
        .route("/api/v1/declarations/{id}/amend", post(amend_declaration))
        .route(
            "/api/v1/declarations/{id}/certificate",
            get(get_certificate),
        )
        .route(
            "/api/v1/declarations/{id}/petition-window",
            get(get_petition_window),
        )
        .route(
            "/api/v1/declarations/{id}/judiciary-ack",
            put(judiciary_ack),
        )
        .with_state(state)
}
