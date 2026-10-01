//! Public contract handlers (dereg/appeal.yaml).
//!
//! Response shapes are taken from the contract. The detail worth recording: the
//! previous `decide` handler returned `201 Created` with
//! `reinstated: verdict == "reinstate"` for **any** appeal id, and it had the
//! direction inverted -- the contract's enum is `[upheld, dismissed]`, where
//! upholding the appeal means the deregistration was wrong and the voter is
//! restored. A dismissal is what leaves a person off the register.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// `AppealDecisionCreate`.
#[derive(Debug, Deserialize)]
pub struct AppealDecisionCreate {
    /// `upheld` or `dismissed`.
    pub verdict: String,
    pub grounds: String,
    /// SHA-256(salt||officer_id).
    pub decided_by_hash: String,
}

/// `DeregUSSDRequest`.
#[derive(Debug, Deserialize)]
pub struct DeregUssdRequest {
    pub session_id: String,
    pub msisdn_hash: String,
    /// Accumulated `*`-delimited input; empty on session start.
    pub text: String,
    /// `*384#`.
    pub service_code: String,
}

/// `DeregUSSDResponse`: `action` is `CON` or `END` on this contract.
#[derive(Debug, Serialize)]
pub struct DeregUssdResponse {
    pub response_text: String,
    pub action: String,
}

/// A problem+json response.
fn problem(status: StatusCode, detail: impl Into<String>) -> Response {
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

fn page(items: &serde_json::Value) -> serde_json::Value {
    eel_spec_helpers_rs::page_flat_value(items)
}

/// `GET /api/v1/dereg/appeals` — list deregistration appeals.
pub async fn list_dereg_appeals(State(state): State<AppState>) -> Response {
    // Previously an empty page for every call, so a filed case was invisible.
    let items = state.store.appeals();
    Json(page(&serde_json::json!(items))).into_response()
}

/// `GET /api/v1/dereg/appeals/{appeal_id}` — fetch one appeal.
pub async fn get_dereg_appeal(
    State(state): State<AppState>,
    Path(appeal_id): Path<String>,
) -> Response {
    // Previously 404 for every id, so a filed case could not be read back.
    match state.store.appeal(&appeal_id) {
        Some(appeal) => Json(appeal).into_response(),
        None => problem(StatusCode::NOT_FOUND, format!("appeal {appeal_id} not found")),
    }
}

/// `POST /api/v1/dereg/appeals/{appeal_id}/decisions` — record a decision.
pub async fn decide_dereg_appeal(
    State(state): State<AppState>,
    Path(appeal_id): Path<String>,
    Json(req): Json<AppealDecisionCreate>,
) -> Response {
    // Previously answered `201 Created` with `reinstated: verdict == "reinstate"`
    // for any appeal id: a decision reinstating a voter's registration against a
    // case that need not exist.
    if req.grounds.trim().is_empty() || req.decided_by_hash.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "grounds and decided_by_hash are required: a decision on a deregistration \
             appeal is a finding about a person's registration",
        );
    }
    if req.grounds.chars().count() > 3000 {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "grounds is at most 3000 characters",
        );
    }
    match state.store.decide(
        &appeal_id,
        &req.verdict,
        req.grounds.trim(),
        req.decided_by_hash.trim(),
    ) {
        Ok(decision) => {
            // The contract's `AppealDecision` is allOf(AppealDecisionCreate) plus
            // decision_id, reinstated and decided_at, so all six are returned.
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "decision_id": decision.decision_id,
                    "appeal_id": appeal_id,
                    "verdict": decision.verdict,
                    "grounds": decision.grounds,
                    "decided_by_hash": decision.decided_by_hash,
                    "reinstated": decision.reinstated,
                    "decided_at": decision.decided_at,
                    "blocking_reason": decision.blocking_reason,
                })),
            )
                .into_response()
        }
        Err(detail) if detail.starts_with("no appeal") => {
            problem(StatusCode::NOT_FOUND, detail).into_response()
        }
        Err(detail) if detail.starts_with("verdict must be") => {
            problem(StatusCode::UNPROCESSABLE_ENTITY, detail).into_response()
        }
        Err(detail) => problem(StatusCode::CONFLICT, detail).into_response(),
    }
}

/// `POST /api/v1/dereg/ussd/sessions` — dereg USSD session.
pub async fn handle_dereg_ussd_session(
    State(state): State<AppState>,
    Json(req): Json<DeregUssdRequest>,
) -> Response {
    // Previously answered `END Service unavailable` to every caller: an honest
    // refusal shaped as a session that started and immediately hung up. The menu
    // is now real.
    let session_id = req.session_id.trim();
    if session_id.is_empty() || session_id.len() > 128 {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "session_id must be between 1 and 128 characters",
        );
    }
    if !is_hex64(&req.msisdn_hash) {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "msisdn_hash must be 64 lowercase hex characters: SHA-256(salt||msisdn) computed \
             by telco middleware. The raw MSISDN must not be transmitted.",
        );
    }
    if req.text.len() > 255 {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "text is at most 255 characters",
        );
    }
    if !req.service_code.starts_with('*') || !req.service_code.ends_with('#') {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "service_code must look like *384#",
        );
    }
    let (response_text, action) = state.store.ussd_step(session_id, &req.text);
    Json(DeregUssdResponse { response_text, action }).into_response()
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Composes the public dereg contract router.
pub fn router() -> axum::Router<crate::state::AppState> {
    axum::Router::new()
        .route("/api/v1/dereg/appeals", axum::routing::get(list_dereg_appeals))
        .route(
            "/api/v1/dereg/appeals/{appeal_id}",
            axum::routing::get(get_dereg_appeal),
        )
        .route(
            "/api/v1/dereg/appeals/{appeal_id}/decisions",
            axum::routing::post(decide_dereg_appeal),
        )
        .route(
            "/api/v1/dereg/ussd/sessions",
            axum::routing::post(handle_dereg_ussd_session),
        )
}
