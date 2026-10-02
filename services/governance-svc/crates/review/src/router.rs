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
    routing::{get, patch, post},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/v1/review/findings",
            post(submit_finding).get(list_findings),
        )
        .route("/v1/review/findings/{id}", patch(triage_finding))
        .route("/v1/review/recommendations", get(list_recommendations))
        .route(
            "/v1/review/recommendations/{id}/sign",
            post(sign_recommendation),
        )
        .route("/v1/review/public", get(public_tracker))
        .route(
            "/v1/review/parliament-referrals",
            post(create_parliament_referral),
        )
        .route(
            "/v1/review/parliament-referrals/{id}",
            patch(record_parliament_response),
        )
        .route("/v1/review/long-drip", get(list_long_drip))
        .route(
            "/v1/review/long-drip/{id}/review",
            post(record_long_drip_review),
        )
        .route("/v1/review/next-cycle-handoff", get(next_cycle_handoff))
        // Public contract endpoints (governance-review/post-election.yaml)
        .route(
            "/api/v1/governance/review/reviews",
            get(list_post_election_reviews).post(initiate_post_election_review),
        )
        .route(
            "/api/v1/governance/review/reviews/{review_id}",
            get(get_post_election_review),
        )
        .route(
            "/api/v1/governance/review/reviews/{review_id}/findings",
            post(add_review_finding),
        )
        .route(
            "/api/v1/governance/review/reviews/{review_id}/publish",
            post(publish_review_report),
        )
        .with_state(state)
}
