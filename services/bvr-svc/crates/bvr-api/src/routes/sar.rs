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

//! Subject access requests under the Kenya Data Protection Act 2019, Article 26.
//!
//! The previous handler minted a `sar_id` and answered 202, which left the caller a
//! ticket for a request that was never recorded -- and it took the requester's
//! **raw** national ID, where the contract asks for a hash and class A data should
//! not be written to a ticket table in the clear.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    error::ApiError,
    store::{SarAccepted, SarRequestRow, StoreError, DELIVERY_CHANNELS},
};

/// `SarRequest`.
#[derive(Debug, Deserialize)]
pub struct SarCreateRequest {
    pub voter_id: String,
    /// SHA-256 of the requester's national ID. The raw number is never accepted.
    pub requestor_national_id_hash: String,
    /// email, postal or in_person.
    pub delivery_channel: String,
    pub requestor_email: Option<String>,
    pub notes: Option<String>,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new().route("/api/v1/bvr/sar", post(create_sar))
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

async fn create_sar(
    State(state): State<crate::state::AppState>,
    Json(req): Json<SarCreateRequest>,
) -> Result<Response, ApiError> {
    if !is_hex64(req.requestor_national_id_hash.trim()) {
        return Err(ApiError::BadRequest(
            "requestor_national_id_hash must be 64 lowercase hex characters. The raw \
             national ID number is not accepted: this handler previously took it and \
             wrote it to the ticket table in the clear"
                .to_string(),
        ));
    }
    if !DELIVERY_CHANNELS.contains(&req.delivery_channel.trim()) {
        return Err(ApiError::BadRequest(format!(
            "delivery_channel must be one of {}, got {}",
            DELIVERY_CHANNELS.join(", "),
            req.delivery_channel
        )));
    }
    if let Some(email) = req.requestor_email.as_deref() {
        // A minimal shape check: one @, something either side, a dot in the domain.
        let plausible = email
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
        if !plausible {
            return Err(ApiError::BadRequest("requestor_email is not a valid address".into()));
        }
    }
    if req.notes.as_ref().is_some_and(|n| n.chars().count() > 1000) {
        return Err(ApiError::BadRequest("notes is at most 1000 characters".into()));
    }
    let row = SarRequestRow {
        sar_id: uuid::Uuid::now_v7().to_string(),
        voter_id: req.voter_id.trim().to_string(),
        requestor_national_id_hash: req.requestor_national_id_hash.trim().to_string(),
        delivery_channel: req.delivery_channel.trim().to_string(),
        notes: req.notes,
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };
    let accepted: SarAccepted = state.store.record_sar(row).await.map_err(|e| match e {
        StoreError::Invalid(m) => ApiError::BadRequest(m),
        StoreError::Database(m) => ApiError::Internal(m),
        other => ApiError::Internal(format!("{other:?}")),
    })?;
    Ok((StatusCode::ACCEPTED, Json(accepted)).into_response())
}
