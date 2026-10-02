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
        // M49 referendum endpoints
        .route(
            "/v1/referendums",
            get(list_referendums).post(create_referendum),
        )
        .route("/v1/referendums/{id}", get(get_referendum))
        .route("/v1/referendums/{id}/activate", put(activate_referendum))
        .route(
            "/v1/referendums/{id}/county-results",
            post(submit_county_result),
        )
        .route(
            "/v1/referendums/{id}/county-results/{county}/certify",
            put(certify_county_result),
        )
        .route(
            "/v1/referendums/{id}/national-result",
            get(get_national_result),
        )
        .route("/v1/referendums/{id}/declare", post(declare_referendum))
        // Public transparency endpoints (governance-referendum/mechanics.yaml)
        .route(
            "/api/v1/governance/referendum/questions",
            get(list_referendum_questions).post(register_referendum_question),
        )
        .route(
            "/api/v1/governance/referendum/results",
            get(get_referendum_results),
        )
        .route(
            "/api/v1/governance/referendum/county-threshold",
            get(get_county_threshold_status),
        )
        // Explainer public contract (explainer/referendum.yaml)
        .route(
            "/api/v1/explainer/referendum/questions",
            get(crate::explainer::list_referendum_explainers)
                .post(crate::explainer::publish_referendum_explainer),
        )
        .route(
            "/api/v1/explainer/referendum/questions/{question_id}",
            get(crate::explainer::get_referendum_explainer),
        )
        .route(
            "/api/v1/explainer/referendum/questions/{question_id}/translations",
            get(crate::explainer::get_referendum_explainer_translations),
        )
        .with_state(state)
}
