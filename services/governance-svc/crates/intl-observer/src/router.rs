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
use axum::{routing::get, Router};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/v1/observer-missions",
            get(list_missions).post(register_mission),
        )
        .route(
            "/v1/observer-missions/{id}/briefings",
            get(list_briefings).post(schedule_briefing),
        )
        .route(
            "/v1/observer-missions/{id}/reports",
            get(list_reports).post(submit_report),
        )
        .route(
            "/v1/joint-statements",
            get(list_joint_statements).post(publish_joint_statement),
        )
        // Public contract endpoints (governance-intl-observer/coordination.yaml)
        .route(
            "/api/v1/governance/intl-observer/missions",
            get(list_observer_missions).post(register_observer_mission),
        )
        .route(
            "/api/v1/governance/intl-observer/missions/{mission_id}",
            get(get_observer_mission),
        )
        .route(
            "/api/v1/governance/intl-observer/missions/{mission_id}/reports",
            get(list_observer_reports).post(submit_observer_report),
        )
        .with_state(state)
}
