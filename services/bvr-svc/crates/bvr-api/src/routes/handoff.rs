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

//! The citizen portal's single-use handoff token.
//!
//! This returned `status: "redeemed"` and `voter_id: null` for any token. A handoff
//! token is a **JWT signed by the citizen portal**, and redeeming one is a state
//! change: the token is single-use, so a redeem must be recorded or the same token
//! redeems twice.

use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::Serialize;

use crate::{error::ApiError, state::AppState};

/// The shape the contract asks for on success.
#[derive(Debug, Serialize)]
pub struct HandoffResponse {
    pub token: String,
    pub status: String,
    pub voter_id: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/bvr/handoff/{token}", post(process_handoff))
}

async fn process_handoff(
    State(_state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<HandoffResponse>, ApiError> {
    let _ = token;
    // The token is a JWT signed by the citizen portal, so validating it needs that
    // portal's verification key, and recording the single-use redemption needs a
    // store for it. Neither is in this workspace.
    //
    // This used to answer `status: "redeemed"` for any string, which told a caller
    // their token had been consumed when nothing had been checked and nothing had
    // been recorded.
    Err(ApiError::NotImplemented(
        "a handoff token is a JWT signed by the citizen portal: validating it needs that \
         portal's verification key, and redeeming it is a single-use state change that \
         needs a redemption store. Neither is in this workspace. This handler previously \
         answered status redeemed for any token, which told a caller their token had been \
         consumed when nothing was checked and nothing was recorded"
            .to_string(),
    ))
}
