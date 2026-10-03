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
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/cvr/events", get(list_events))
}

/// GET /v1/cvr/events
///
/// Most recent voter-register events from `eel_voter_register_events`
/// (created by migration V050), newest first.
async fn list_events(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let rows: Vec<(
        Uuid,
        Uuid,
        String,
        String,
        Option<f32>,
        String,
        Option<DateTime<Utc>>,
        DateTime<Utc>,
    )> = sqlx::query_as(
        "SELECT event_id, voter_id, event_type, source, confidence_score, decision, \
         resolved_at, created_at \
         FROM eel_voter_register_events \
         ORDER BY created_at DESC \
         LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let events: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                event_id,
                voter_id,
                event_type,
                source,
                confidence_score,
                decision,
                resolved_at,
                created_at,
            )| {
                json!({
                    "eventId": event_id,
                    "voterId": voter_id,
                    "eventType": event_type,
                    "source": source,
                    "confidenceScore": confidence_score,
                    "decision": decision,
                    "resolvedAt": resolved_at,
                    "createdAt": created_at,
                })
            },
        )
        .collect();

    Ok(Json(json!({ "events": events })))
}
