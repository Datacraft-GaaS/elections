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

//! Route wiring for every operation in the BVI contract.

pub mod anomalies;
pub mod canaries;
pub mod config;
pub mod identifications;
pub mod intake_guard;
pub mod kits;
pub mod register_anchors;
pub mod sessions;
pub mod signals;
pub mod turnout;
pub mod voters;

use crate::{
    error::ApiError,
    state::AppState,
    store::{BviStore, StoreError},
};
use axum::{routing::get, routing::patch, routing::post, Router};

/// Full contract router. Paths mirror packages/openapi-contracts/bvi/openapi.yaml.
pub fn router<S: BviStore>(state: AppState<S>) -> Router {
    Router::new()
        .route("/api/v1/bvi/sessions", post(sessions::open_bvi_session))
        .route(
            "/api/v1/bvi/sessions/{id}/close",
            patch(sessions::close_bvi_session),
        )
        .route(
            "/api/v1/bvi/identifications",
            post(identifications::record_identification),
        )
        .route("/api/v1/bvi/telemetry", post(signals::submit_telemetry))
        .route("/api/v1/bvi/distress", post(signals::raise_distress))
        .route(
            "/api/v1/bvi/config/{station_code}",
            get(config::get_station_config),
        )
        .route("/api/v1/bvi/kits/{serial}/revoke", post(kits::revoke_kit))
        .route("/api/v1/bvi/kits/health", get(kits::list_kits_health))
        .route("/api/v1/bvi/turnout", get(turnout::get_turnout))
        .route(
            "/api/v1/bvi/voters/{voter_id}/station",
            get(voters::get_voter_station),
        )
        .route("/api/v1/bvi/anomalies/evaluate", post(anomalies::evaluate))
        .route(
            "/api/v1/bvi/anomalies/canary-hits",
            get(canaries::get_canary_hits),
        )
        .route(
            "/api/v1/bvi/register/anchors",
            get(register_anchors::get_register_anchors),
        )
        .with_state(state)
}

/// Map a generic StoreError onto the problem+json surface for an operation.
pub(crate) fn store_error(e: StoreError, instance: &'static str) -> ApiError {
    match e {
        StoreError::NotFound => ApiError::not_found(instance),
        StoreError::Conflict(detail) => ApiError::conflict(instance, detail),
        _ => ApiError::internal(instance),
    }
}
