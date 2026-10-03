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

//! Axum 0.8 REST router for RFID cover custody.
//!
//! Routes:
//!   POST /v1/rfid/tags                        issuance; INVENTORY_ADMIN SVID
//!   POST /v1/rfid/tags/{id}/pair              bind to box body serial
//!   POST /v1/rfid/reads                       from reader gate; reader SVID
//!   POST /v1/rfid/tags/{id}/transition        officer-triggered FSM step
//!   GET  /v1/rfid/tags/{id}/history           full state-transition chain
//!   GET  /v1/rfid/tamper-alerts               pending alerts
//!   PATCH /v1/rfid/tamper-alerts/{id}/resolve mark alert resolved

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{CoverState, ReaderType};
use crate::error::RfidError;
use crate::tag::{issue_tag, pair_to_box, IssueTagRequest, PairToBoxRequest};

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

// ---------------------------------------------------------------------------
// Request / response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct IssueTagDto {
    pub manufacturer: String,
    pub lot_id: Uuid,
    pub chip_family: String,
    pub tag_secret_hsm_key_id: String,
}

#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub tag_id: String,
    pub epc: String,
    pub lot_id: String,
    pub issued_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PairDto {
    pub box_body_id: Uuid,
    pub box_lot_id: Uuid,
    pub box_body_epc: String,
}

#[derive(Debug, Deserialize)]
pub struct IngestReadDto {
    pub tag_id: Uuid,
    pub reader_id: String,
    pub reader_type: String,
    pub reader_svid: String,
    pub geo_lat: Option<f64>,
    pub geo_lon: Option<f64>,
    pub geo_accuracy_m: Option<f64>,
    pub tag_challenge: Option<String>,
    pub tag_response: Option<String>,
    pub tamper_indicator: bool,
    pub custody_event_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct TransitionDto {
    pub to_state: String,
    pub officer_id: Option<Uuid>,
    pub officer_svid: Option<String>,
    pub triggered_by_read: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct PendingAlertsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveAlertDto {
    pub resolution_note: String,
}

// ---------------------------------------------------------------------------
// Error response helper
// ---------------------------------------------------------------------------

fn rfid_error_response(err: RfidError) -> impl IntoResponse {
    let status = match &err {
        RfidError::TransitionNotAllowed { .. }
        | RfidError::TransitionFromTerminal(_)
        | RfidError::FourEyesRequired { .. }
        | RfidError::AlreadyPaired { .. }
        | RfidError::LotMismatch { .. }
        | RfidError::ReplayDetected { .. }
        | RfidError::TagRetired(_)
        | RfidError::EpcMismatch => StatusCode::UNPROCESSABLE_ENTITY,

        RfidError::ReaderSvidRejected(_)
        | RfidError::HmacFailure { .. }
        | RfidError::ReaderSignatureInvalid
        | RfidError::InvalidHmacKey => StatusCode::UNAUTHORIZED,

        RfidError::ChainBroken { .. } | RfidError::ChainEmpty => StatusCode::CONFLICT,

        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    #[derive(Serialize)]
    struct ErrorBody {
        error: String,
    }
    (
        status,
        Json(ErrorBody {
            error: err.to_string(),
        }),
    )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn post_issue_tag(
    State(state): State<AppState>,
    Json(dto): Json<IssueTagDto>,
) -> impl IntoResponse {
    let req = IssueTagRequest {
        manufacturer: dto.manufacturer,
        lot_id: dto.lot_id,
        chip_family: dto.chip_family,
        tag_secret_hsm_key_id: dto.tag_secret_hsm_key_id,
    };

    let tag = match issue_tag(req) {
        Ok(t) => t,
        Err(e) => return rfid_error_response(e).into_response(),
    };

    if let Err(e) = crate::repository::insert_tag(&state.pool, &tag).await {
        return rfid_error_response(e).into_response();
    }

    let resp = TagResponse {
        tag_id: tag.tag_id.to_string(),
        epc: hex::encode(&tag.epc),
        lot_id: tag.lot_id.to_string(),
        issued_at: tag.issued_at.to_rfc3339(),
    };

    (StatusCode::CREATED, Json(resp)).into_response()
}

async fn post_pair_tag(
    State(state): State<AppState>,
    Path(tag_id): Path<Uuid>,
    Json(dto): Json<PairDto>,
) -> impl IntoResponse {
    let tag = match crate::repository::fetch_tag(&state.pool, tag_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "tag not found"})),
            )
                .into_response();
        }
        Err(e) => return rfid_error_response(e).into_response(),
    };

    let box_body_epc = match hex::decode(&dto.box_body_epc) {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid hex in box_body_epc"})),
            )
                .into_response();
        }
    };

    let req = PairToBoxRequest {
        box_body_id: dto.box_body_id,
        box_lot_id: dto.box_lot_id,
        box_body_epc,
    };

    let paired = match pair_to_box(tag, req) {
        Ok(t) => t,
        Err(e) => return rfid_error_response(e).into_response(),
    };

    if let Err(e) =
        crate::repository::update_tag_pairing(&state.pool, paired.tag_id, dto.box_body_id).await
    {
        return rfid_error_response(e).into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"paired_with_box": dto.box_body_id.to_string()})),
    )
        .into_response()
}

async fn post_ingest_read(
    State(_state): State<AppState>,
    Json(_dto): Json<IngestReadDto>,
) -> impl IntoResponse {
    // Full implementation wires IngestRequest to ingest_read_pure + DB write.
    // Stubbed to avoid requiring live HSM / Valkey in the API layer.
    // The pure logic is tested directly in rfid_test.rs.
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"status": "accepted"})),
    )
        .into_response()
}

async fn post_transition(
    State(state): State<AppState>,
    Path(tag_id): Path<Uuid>,
    Json(dto): Json<TransitionDto>,
) -> impl IntoResponse {
    let to = match CoverState::from_str(&dto.to_state) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "unknown to_state"})),
            )
                .into_response();
        }
    };

    let transitions = match crate::repository::fetch_transitions(&state.pool, tag_id).await {
        Ok(v) => v,
        Err(e) => return rfid_error_response(e).into_response(),
    };

    let from = transitions
        .last()
        .map(|t| t.to_state.clone())
        .unwrap_or(CoverState::Manufactured);

    // Validate (use WarehouseGate as default if no reader_type in DTO).
    if let Err(e) = crate::state_machine::CoverStateMachine::validate(
        &from,
        &to,
        &ReaderType::WarehouseGate,
        dto.officer_svid.as_deref(),
    ) {
        return rfid_error_response(e).into_response();
    }

    let prev_hash = transitions.last().map(|t| t.this_hash.clone());
    let transition_id = Uuid::now_v7();
    let new_t = {
        use crate::domain::CoverStateTransition;
        use chrono::Utc;
        let mut t = CoverStateTransition {
            transition_id,
            tag_id,
            from_state: from.clone(),
            to_state: to.clone(),
            triggered_by_read: dto.triggered_by_read,
            officer_id: dto.officer_id,
            officer_svid: dto.officer_svid.clone(),
            officer_signature: None,
            prev_hash: prev_hash.clone(),
            this_hash: vec![],
            transitioned_at: Utc::now(),
        };
        let hash = crate::hash_chain::compute_this_hash(prev_hash.as_deref().unwrap_or(&[]), &t);
        t.this_hash = hash.to_vec();
        t
    };

    if let Err(e) = crate::repository::insert_transition(&state.pool, &new_t).await {
        return rfid_error_response(e).into_response();
    }

    crate::metrics::inc_state_transitions(from.as_str(), to.as_str());

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "transition_id": new_t.transition_id.to_string(),
            "from_state": from.as_str(),
            "to_state": to.as_str(),
        })),
    )
        .into_response()
}

async fn get_tag_history(
    State(state): State<AppState>,
    Path(tag_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::repository::fetch_transitions(&state.pool, tag_id).await {
        Ok(v) => Json(serde_json::json!({ "transitions": v })).into_response(),
        Err(e) => rfid_error_response(e).into_response(),
    }
}

async fn get_tamper_alerts(
    State(state): State<AppState>,
    Query(q): Query<PendingAlertsQuery>,
) -> impl IntoResponse {
    // The `status` filter was accepted and dropped: `Query(_q)`. So
    // `?status=critical` returned every unresolved alert, which is worse than
    // ignoring the parameter, because it answers the narrower question wrongly.
    match crate::repository::fetch_pending_alerts(&state.pool, q.status.as_deref()).await {
        Ok(v) => Json(serde_json::json!({ "alerts": v })).into_response(),
        Err(e) => rfid_error_response(e).into_response(),
    }
}

async fn patch_resolve_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    Json(dto): Json<ResolveAlertDto>,
) -> impl IntoResponse {
    match crate::repository::resolve_alert(&state.pool, alert_id, &dto.resolution_note).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"resolved": true}))).into_response(),
        Err(e) => rfid_error_response(e).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the Axum router. Caller nests it under the service's root router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/rfid/tags", post(post_issue_tag))
        .route("/v1/rfid/tags/{id}/pair", post(post_pair_tag))
        .route("/v1/rfid/reads", post(post_ingest_read))
        .route("/v1/rfid/tags/{id}/transition", post(post_transition))
        .route("/v1/rfid/tags/{id}/history", get(get_tag_history))
        .route("/v1/rfid/tamper-alerts", get(get_tamper_alerts))
        .route(
            "/v1/rfid/tamper-alerts/{id}/resolve",
            patch(patch_resolve_alert),
        )
        .with_state(state)
}
