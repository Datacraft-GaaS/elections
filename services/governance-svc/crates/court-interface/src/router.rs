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
    routing::{get, post},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/court/petitions", get(list_petitions).post(file_petition))
        .route("/court/petitions/{petition_id}", get(get_petition))
        .route(
            "/court/petitions/{petition_id}/evidence/request",
            post(request_evidence),
        )
        .route(
            "/court/petitions/{petition_id}/evidence/bundle",
            get(get_evidence_bundle),
        )
        .route(
            "/court/petitions/{petition_id}/witnesses",
            get(list_witnesses).post(register_witness),
        )
        .route(
            "/court/petitions/{petition_id}/supplementary",
            post(add_supplementary_request),
        )
        // Public contract endpoints (governance-court/interface.yaml)
        .route(
            "/api/v1/governance/court/petitions",
            get(list_petitions).post(file_petition),
        )
        .route(
            "/api/v1/governance/court/petitions/{petition_id}",
            get(get_petition),
        )
        .route(
            "/api/v1/governance/court/petitions/{petition_id}/evidence-requests",
            post(create_evidence_request),
        )
        .route(
            "/api/v1/governance/court/petitions/{petition_id}/judgments",
            post(record_judgment),
        )
        .with_state(state)
}
