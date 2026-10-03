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

//! Public contract handlers (voter-card/issuance.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::store::StoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterCardRequest {
    pub request_id: String,
    pub status: String,
    pub submitted_at: String,
}

#[derive(Debug, Deserialize)]
pub struct VoterCardRequestCreate {
    pub voter_id_hash: String,
    pub request_type: String,
    pub reason: String,
    pub preferred_collection_centre: String,
}

#[derive(Debug, Deserialize)]
pub struct CollectionConfirmation {
    pub collected_at: String,
    pub collection_centre_code: String,
    pub officer_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionCentre {
    pub centre_code: String,
    pub name: String,
    pub county_code: String,
    pub address: String,
    pub operating_hours: Option<String>,
    pub open: bool,
}

fn store_err(e: StoreError) -> axum::response::Response {
    match e {
        StoreError::NotFound => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "not found"})),
        )
            .into_response(),
        StoreError::Db(err) => {
            tracing::error!(error = %err, "store db error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "detail": "internal error"})),
            )
                .into_response()
        }
    }
}

/// POST /api/v1/voter-card/requests — request a voter card (postgres).
pub async fn request_voter_card(
    State(state): State<AppState>,
    Json(req): Json<VoterCardRequestCreate>,
) -> impl IntoResponse {
    match state.store.create_request(req).await {
        Ok(rec) => (StatusCode::CREATED, Json(rec)).into_response(),
        Err(e) => store_err(e),
    }
}

/// GET /api/v1/voter-card/requests/{request_id} — fetch a request (postgres).
pub async fn get_voter_card_request(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
) -> impl IntoResponse {
    match state.store.get_request(&request_id).await {
        Ok(Some(rec)) => Json(rec).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": format!("request {request_id} not found") })),
        )
            .into_response(),
        Err(e) => store_err(e),
    }
}

/// POST /api/v1/voter-card/requests/{request_id}/collection-confirmation (postgres).
pub async fn confirm_voter_card_collection(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
    Json(req): Json<CollectionConfirmation>,
) -> impl IntoResponse {
    match state.store.confirm_collection(&request_id, req).await {
        Ok(Some(rec)) => Json(rec).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": format!("request {request_id} not found") })),
        )
            .into_response(),
        Err(e) => store_err(e),
    }
}

/// GET /api/v1/voter-card/collection-centres — list collection centres (postgres).
pub async fn list_collection_centres(State(state): State<AppState>) -> impl IntoResponse {
    match state.store.list_centres().await {
        Ok(centres) => {
            Json(serde_json::json!({ "centres": centres, "total": centres.len() })).into_response()
        }
        Err(e) => store_err(e),
    }
}

/// Composes the public voter-card contract router.
pub fn router() -> axum::Router<crate::state::AppState> {
    axum::Router::new()
        .route(
            "/api/v1/voter-card/requests",
            axum::routing::post(request_voter_card),
        )
        .route(
            "/api/v1/voter-card/requests/{request_id}",
            axum::routing::get(get_voter_card_request),
        )
        .route(
            "/api/v1/voter-card/requests/{request_id}/collection-confirmation",
            axum::routing::post(confirm_voter_card_collection),
        )
        .route(
            "/api/v1/voter-card/collection-centres",
            axum::routing::get(list_collection_centres),
        )
}
