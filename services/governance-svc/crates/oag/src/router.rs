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
        .route(
            "/oag/engagements",
            get(list_engagements).post(create_engagement),
        )
        .route("/oag/engagements/{engagement_id}", get(get_engagement))
        .route(
            "/oag/engagements/{engagement_id}/evidence",
            post(extract_evidence),
        )
        .route(
            "/oag/engagements/{engagement_id}/response",
            post(record_response),
        )
        .route(
            "/oag/engagements/{engagement_id}/response",
            get(get_response),
        )
        .route(
            "/oag/published-reports",
            get(list_published_reports).post(register_published_report),
        )
        .with_state(state)
}
