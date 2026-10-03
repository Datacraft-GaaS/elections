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

use crate::handlers::*;
use crate::state::SharedState;
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        // M50 by-election endpoints
        .route(
            "/v1/by-elections",
            get(list_by_elections).post(create_by_election),
        )
        .route("/v1/by-elections/{id}", get(get_by_election))
        .route(
            "/v1/by-elections/{id}/register-subset",
            post(generate_register_subset),
        )
        .route("/v1/by-elections/{id}/kit-plan", post(create_kit_plan))
        .route("/v1/by-elections/{id}/activate", put(activate_by_election))
        .route(
            "/v1/by-elections/{id}/cost-report",
            post(submit_cost_report).get(get_cost_report),
        )
        // Public contract endpoints (governance-by-election/operations.yaml)
        .route(
            "/api/v1/governance/by-election/elections",
            get(list_by_elections).post(declare_by_election),
        )
        .route(
            "/api/v1/governance/by-election/elections/{by_election_id}",
            get(get_by_election),
        )
        .route(
            "/api/v1/governance/by-election/elections/{by_election_id}/status",
            axum::routing::patch(transition_by_election_status),
        )
        .with_state(state)
}
