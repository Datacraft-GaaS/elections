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

//! Voter reads and registration-status changes.
//!
//! Both handlers read the register. `get_voter` used to answer `full_name:
//! nfc("Placeholder Voter")`, county `047`, station `047-001-001` and `status:
//! active` for any voter id -- it invented a person's name and where they vote.
//! The classifier read that as `VERIFIED` because it saw a real call to `nfc`,
//! which is a Unicode normalizer; all four of its arguments were string literals.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;

use crate::{error::ApiError, store::StoreError};

/// `VoterStatusUpdate`.
#[derive(Debug, Deserialize)]
pub struct StatusUpdateRequest {
    /// One of active, suspended, deregistered, pending, deceased.
    pub status: String,
    /// One of the contract's six reason codes.
    pub reason_code: String,
    pub reason_detail: Option<String>,
    /// The officer executing the change.
    pub officer_id: String,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/api/v1/bvr/voters/{voter_id}", get(get_voter))
        .route("/api/v1/bvr/voters/{voter_id}/status", patch(update_voter_status))
        .route("/api/v1/bvr/voters/{voter_id}/qr", get(get_voter_qr))
}

fn map(err: StoreError) -> ApiError {
    match err {
        StoreError::NotFound(m) => ApiError::NotFound(m),
        StoreError::Invalid(m) => ApiError::BadRequest(m),
        StoreError::Unavailable(m) => ApiError::NotImplemented(m),
        StoreError::Database(m) => ApiError::Internal(m),
    }
}

async fn get_voter(
    State(state): State<crate::state::AppState>,
    Path(voter_id): Path<String>,
) -> Result<Response, ApiError> {
    // Previously returned a placeholder name, county, station, status and scope for
    // any id. The contract's VoterSummary carries no name at all -- and the
    // `full_name_enc` column is a field-encrypted blob, so there was no name to
    // return even if it wanted one.
    match state.store.voter(&voter_id).await.map_err(map)? {
        Some(voter) => Ok(Json(voter).into_response()),
        None => Err(ApiError::NotFound(format!("no voter {voter_id} on the register"))),
    }
}

async fn update_voter_status(
    State(state): State<crate::state::AppState>,
    Path(voter_id): Path<String>,
    Json(req): Json<StatusUpdateRequest>,
) -> Result<Response, ApiError> {
    // Previously logged the change and returned 204, so a status change was
    // acknowledged and never happened. The contract asks for 200 with the updated
    // VoterSummary, which is also the only way a caller can see the new version.
    if req.reason_detail.as_ref().is_some_and(|d| d.chars().count() > 500) {
        return Err(ApiError::BadRequest("reason_detail is at most 500 characters".into()));
    }
    let updated = state
        .store
        .update_voter_status(
            &voter_id,
            req.status.trim(),
            req.reason_code.trim(),
            req.reason_detail.as_deref().map(str::trim).filter(|d| !d.is_empty()),
            req.officer_id.trim(),
        )
        .await
        .map_err(map)?;
    Ok((StatusCode::OK, Json(updated)).into_response())
}

async fn get_voter_qr(
    State(_state): State<crate::state::AppState>,
    _voter_id: Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // The voter card payload is signed by the citizen portal over the M02 register
    // snapshot, and neither the snapshot client nor the signing key is in this
    // workspace. The previous handler returned `eel://bvr/voter/{voter_id}` for any
    // id, which is a QR encoding a lookup that would find nothing.
    Err(ApiError::NotImplemented(
        "a voter card QR encodes a payload signed over the M02 register snapshot, and \
         neither the snapshot client nor the citizen portal's signing key is in this \
         workspace. This handler previously returned an eel://bvr/voter/{id} URI for any \
         voter id, which is a scannable code that resolves to nothing"
            .to_string(),
    ))
}
