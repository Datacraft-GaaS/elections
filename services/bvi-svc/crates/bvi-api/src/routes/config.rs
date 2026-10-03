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

//! GET /api/v1/bvi/config/{station_code}.

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::{
    error::ApiError, models::StationConfigResponse, routes::store_error, state::AppState,
    store::BviStore,
};

const INSTANCE: &str = "/api/v1/bvi/config/{station_code}";

#[derive(Debug, Deserialize)]
pub struct ConfigQuery {
    pub election_id: Option<String>,
}

/// GET /api/v1/bvi/config/{station_code} (getStationConfig).
///
/// Devices pull config at boot; responses are cacheable for 300s and served
/// with a strong ETag derived from the config version (304 on match).
pub async fn get_station_config<S: BviStore>(
    State(state): State<AppState<S>>,
    Path(station_code): Path<String>,
    Query(q): Query<ConfigQuery>,
) -> Result<Response, ApiError> {
    if q.election_id.as_deref().map(str::is_empty).unwrap_or(true) {
        return Err(ApiError::bad_request(INSTANCE, "election_id is required"));
    }

    let config = state
        .store
        .station_config(&station_code)
        .await
        .map_err(|e| store_error(e, INSTANCE))?
        .ok_or_else(|| ApiError::not_found(INSTANCE))?;

    let etag = format!("\"cfg-v{}-{}\"", config.version, station_code);

    let body = StationConfigResponse {
        station_code: config.station_code,
        election_id: q.election_id.unwrap_or_default(),
        match_threshold: config.match_threshold,
        allowed_modalities: config.allowed_modalities,
        timeout_s: config.timeout_s,
        fallback_allowed: config.fallback_allowed,
    };

    Ok((
        StatusCode::OK,
        [
            (header::CACHE_CONTROL, "max-age=300, private"),
            (header::ETAG, etag.as_str()),
        ],
        Json(body),
    )
        .into_response())
}
