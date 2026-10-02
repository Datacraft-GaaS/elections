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

use axum::{extract::{Query, State}, routing::get, Json, Router};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct StatusQuery {
    pub card_number: Option<String>,
    #[allow(dead_code)]
    pub voter_id: Option<uuid::Uuid>,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new().route("/v1/voter-cards/status", get(card_status))
}

/// Card status lookup against the authoritative `card_blacklist` table.
///
/// The full card-registry projection (issued/dispatched/collected states,
/// voter_id join) is part of M53 issuance and is feature-flagged behind the
/// printer integration; blacklist state is live today because BVI edge nodes
/// gate on it during polling-day identification.
async fn card_status(
    State(state): State<crate::state::AppState>,
    Query(q): Query<StatusQuery>,
) -> Result<Json<serde_json::Value>, crate::error::ApiError> {
    let Some(card_number) = q.card_number.filter(|c| !c.is_empty()) else {
        return Err(crate::error::ApiError::Internal(
            "card_number query parameter is required".into(),
        ));
    };

    let row: Option<(String, chrono::DateTime<chrono::Utc>, String)> = sqlx::query_as(
        "SELECT reason, blacklisted_at, source FROM card_blacklist WHERE card_number = $1",
    )
    .bind(&card_number)
    .fetch_optional(state.pool())
    .await
    .map_err(|e| crate::error::ApiError::Internal(e.to_string()))?;

    match row {
        Some((reason, blacklisted_at, source)) => Ok(Json(json!({
            "card_number": card_number,
            "blacklisted": true,
            "reason": reason,
            "source": source,
            "blacklisted_at": blacklisted_at.to_rfc3339(),
        }))),
        None => Ok(Json(json!({
            "card_number": card_number,
            "blacklisted": false,
        }))),
    }
}
