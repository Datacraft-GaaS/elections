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
        .route("/v1/forensics/cycles", get(list_cycles).post(create_cycle))
        .route("/v1/forensics/cycles/{id}/run", put(trigger_run))
        .route("/v1/forensics/cycles/{id}/estimates", get(list_estimates))
        .route("/v1/forensics/cycles/{id}/referrals", post(create_referral))
        .route(
            "/v1/forensics/cycles/{id}/replication",
            post(publish_replication),
        )
        .route("/v1/forensics/reports", get(list_public_reports))
        .route("/v1/forensics/reports/{id}", get(get_public_report))
        // Public contract endpoints (governance-forensics/forensics.yaml)
        .route(
            "/api/v1/governance/forensics/investigations",
            get(list_forensic_investigations).post(initiate_forensic_investigation),
        )
        .route(
            "/api/v1/governance/forensics/investigations/{investigation_id}",
            get(get_forensic_investigation),
        )
        .route(
            "/api/v1/governance/forensics/investigations/{investigation_id}/evidence",
            post(attach_evidence),
        )
        .route(
            "/api/v1/governance/forensics/investigations/{investigation_id}/findings",
            post(add_forensic_finding),
        )
        .with_state(state)
}
