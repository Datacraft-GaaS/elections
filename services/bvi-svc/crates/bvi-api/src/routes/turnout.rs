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

//! GET /api/v1/bvi/turnout.

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bvi_turnout::resolve_scope;
use serde::Deserialize;

use crate::{
    error::ApiError,
    idem,
    routes::store_error,
    state::AppState,
    store::BviStore,
};

const INSTANCE: &str = "/api/v1/bvi/turnout";

#[derive(Debug, Deserialize)]
pub struct TurnoutParams {
    pub election_id: Option<String>,
    pub county_code: Option<String>,
    pub constituency_code: Option<String>,
    pub station_code: Option<String>,
}

/// GET /api/v1/bvi/turnout (getTurnout).
///
/// Aggregate turnout only (data-class D). Scope narrows to the most specific
/// parameter provided; omitting all returns national totals.
pub async fn get_turnout<S: BviStore>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    Query(p): Query<TurnoutParams>,
) -> Result<Response, ApiError> {
    idem::require_bearer(&headers, INSTANCE)?;

    let election_id = p
        .election_id
        .filter(|e| !e.is_empty())
        .ok_or_else(|| ApiError::bad_request(INSTANCE, "election_id is required"))?;

    let scope = resolve_scope(
        p.county_code.as_deref(),
        p.constituency_code.as_deref(),
        p.station_code.as_deref(),
    );

    let summary = state
        .store
        .turnout_summary(&election_id, &scope)
        .await
        .map_err(|e| store_error(e, INSTANCE))?;

    Ok((
        StatusCode::OK,
        [(header::CACHE_CONTROL, "public, max-age=30")],
        Json(summary),
    )
        .into_response())
}
