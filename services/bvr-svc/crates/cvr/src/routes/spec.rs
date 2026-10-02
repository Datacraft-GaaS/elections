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

//! Public contract handlers (cvr/register.yaml).

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct VoterLookup {
    pub id_number_hash: String,
    pub station_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VoterLookupResult {
    pub registered: bool,
    pub polling_station_name: Option<String>,
    pub ward: Option<String>,
    pub constituency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterStats {
    pub total_registered: i64,
    pub counties: i64,
    pub as_of: String,
}

#[derive(Debug, Deserialize)]
pub struct UssdSessionRequest {
    pub session_id: String,
    pub msisdn_hash: String,
    pub text: String,
    pub service_code: String,
}

#[derive(Debug, Serialize)]
pub struct UssdSessionResponse {
    pub response_text: String,
    pub action: String,
}

/// POST /api/v1/cvr/lookups — verify a voter's registration.
pub async fn voter_lookup(
    State(_state): State<AppState>,
    Json(req): Json<VoterLookup>,
) -> ApiError {
    // Previously returned `registered: false` with null station, ward and
    // constituency for **every** request. That is the most dangerous shape in this
    // tree: a definitive-looking negative about a real person's registration, on
    // the one endpoint whose whole job is to answer that question.
    let _ = req;
    ApiError::NotImplemented(crate::store::REGISTER_UNAVAILABLE.to_string())
}

/// GET /api/v1/cvr/register/snapshot — download the register snapshot.
pub async fn get_register_snapshot(State(_state): State<AppState>) -> ApiError {
    // Previously 404 "no snapshot available" for every request, which reads as no
    // snapshot having been published rather than as this service being unable to
    // say.
    ApiError::NotImplemented(crate::store::SNAPSHOT_UNAVAILABLE.to_string())
}

/// GET /api/v1/cvr/register/stats — register statistics.
pub async fn get_register_stats(State(_state): State<AppState>) -> ApiError {
    // Previously returned `total_registered: 0, counties: 0`, which reads as a
    // measured register that happens to be empty.
    ApiError::NotImplemented(crate::store::STATS_UNAVAILABLE.to_string())
}

/// POST /api/v1/cvr/ussd/sessions — handle a USSD session.
pub async fn handle_ussd_session(
    State(_state): State<AppState>,
    Json(_req): Json<UssdSessionRequest>,
) -> impl IntoResponse {
    Json(UssdSessionResponse {
        response_text: "END Service unavailable".to_string(),
        action: "end".to_string(),
    })
}


/// Composes the public cvr contract router.
pub fn router() -> axum::Router<crate::state::AppState> {
	axum::Router::new()
		.route("/api/v1/cvr/lookups", axum::routing::post(voter_lookup))
		.route("/api/v1/cvr/register/snapshot", axum::routing::get(get_register_snapshot))
		.route("/api/v1/cvr/register/stats", axum::routing::get(get_register_stats))
		.route("/api/v1/cvr/ussd/sessions", axum::routing::post(handle_ussd_session))
}
