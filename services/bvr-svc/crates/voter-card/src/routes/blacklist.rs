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

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

use crate::{
    blacklist::BlacklistService, error::ApiError, models::BlacklistEntry, state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/voter-cards/blacklist", post(add_blacklist))
}

async fn add_blacklist(
    State(state): State<AppState>,
    Json(entry): Json<BlacklistEntry>,
) -> Result<StatusCode, ApiError> {
    let svc = BlacklistService;
    svc.add_entry(state.pool(), entry)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::CREATED)
}
