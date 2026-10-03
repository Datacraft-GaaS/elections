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

use crate::{error::ApiError, state::AppState};
use axum::{extract::State, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

/// Row shape of `register_snapshots`, matching the SELECT in
/// [`list_snapshots`].
type SnapshotRow = (
    String,
    String,
    i32,
    i64,
    String,
    String,
    Option<String>,
    DateTime<Utc>,
);

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/cvr/snapshots", get(list_snapshots))
}

/// GET /v1/cvr/snapshots
///
/// Most recent register snapshots from the `register_snapshots` table
/// (created by migration V3), newest first.
async fn list_snapshots(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let rows: Vec<SnapshotRow> = sqlx::query_as(
        "SELECT snapshot_id, election_scope, station_count, voter_count, merkle_root, \
         signed_by, checkpoint, created_at \
         FROM register_snapshots \
         ORDER BY created_at DESC \
         LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let snapshots: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                snapshot_id,
                election_scope,
                station_count,
                voter_count,
                merkle_root,
                signed_by,
                checkpoint,
                created_at,
            )| {
                json!({
                    "snapshotId": snapshot_id,
                    "electionScope": election_scope,
                    "stationCount": station_count,
                    "voterCount": voter_count,
                    "merkleRoot": merkle_root,
                    "signedBy": signed_by,
                    "checkpoint": checkpoint,
                    "createdAt": created_at,
                })
            },
        )
        .collect();

    Ok(Json(json!({ "snapshots": snapshots })))
}
