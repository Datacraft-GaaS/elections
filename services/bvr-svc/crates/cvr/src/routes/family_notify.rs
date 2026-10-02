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

use crate::{error::ApiError, models::FamilyNotificationRequest, state::AppState};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};


pub fn router() -> Router<AppState> {
    Router::new().route("/v1/cvr/family-notify", post(family_notify))
}

async fn family_notify(
    State(_state): State<AppState>,
    Json(_req): Json<FamilyNotificationRequest>,
) -> Result<StatusCode, ApiError> {
    // Persistence prerequisite: FamilyNotificationRequest carries a hashed
    // voter id (SHA-256 hex), but eel_voter_register_events.voter_id is UUID
    // NOT NULL and the voters table (V1) keys on TEXT voter_id with the
    // national id stored encrypted, so there is no lookup path from the hash
    // to an event row. The bvr-svc service migrations (owner: bvr-svc
    // maintainers; next free slot V054) must add either a hashed-voter-id
    // index or a dedicated eel_family_notifications table before the
    // FAMILY_NOTIFICATION / FAMILY_SHORTCODE event can be written here.
    Ok(StatusCode::ACCEPTED)
}
