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

use axum::{routing::get, routing::post, Router};

use crate::handlers::{health, post_snapshot};
use crate::spec_handlers;
use crate::spec_state::SharedState;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/v1/queue/health", get(health))
        .route(
            "/v1/queue/stations/{station_code}/snapshot",
            post(post_snapshot),
        )
        // Public contract endpoints (queue/transparency.yaml)
        .route("/queue/stations", get(spec_handlers::list_station_queue_statuses))
        .route(
            "/queue/stations/{station_code}/status",
            get(spec_handlers::get_station_queue_status),
        )
        .route("/queue/observations", post(spec_handlers::submit_queue_observation))
        .route(
            "/queue/analytics/county/{county_code}",
            get(spec_handlers::get_county_queue_analytics),
        )
        .with_state(state)
}
