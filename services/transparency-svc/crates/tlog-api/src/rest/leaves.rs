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
    Json,
};
use serde::Serialize;

use crate::state::ApiState;

#[derive(Serialize)]
pub struct LeafResponse {
    pub index: i64,
    pub domain: String,
    pub hash: String,
    pub producer_spiffe_id: String,
    pub source_event_id: String,
    pub event_time: String,
}

pub async fn get_leaf(
    State(state): State<ApiState>,
    Path(index): Path<i64>,
) -> Result<Json<LeafResponse>, StatusCode> {
    let row: Option<(
        i64,
        String,
        Vec<u8>,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "SELECT index, domain, hash, producer_spiffe_id, source_event_id, event_time \
             FROM leaves WHERE index=$1",
    )
    .bind(index)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        None => Err(StatusCode::NOT_FOUND),
        Some((idx, domain, hash, producer, event_id, event_time)) => Ok(Json(LeafResponse {
            index: idx,
            domain,
            hash: hex::encode(hash),
            producer_spiffe_id: producer,
            source_event_id: event_id,
            event_time: event_time.to_rfc3339(),
        })),
    }
}
