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
            "/reporting/reports",
            get(list_reports).post(generate_report),
        )
        .route("/reporting/reports/{report_id}", get(get_report))
        .route("/reporting/reports/{report_id}/ready", post(mark_report_ready))
        .route(
            "/reporting/testimony-workspaces",
            get(list_testimony_workspaces).post(create_testimony_workspace),
        )
        .route(
            "/reporting/judiciary-handovers",
            post(record_judiciary_handover),
        )
        .route("/reporting/sc-preregistrations", post(register_sc_it))
    // Public contract endpoints (governance-reporting/reports.yaml)
        .route(
            "/api/v1/governance/reporting/parliamentary-reports",
            get(list_parliamentary_reports).post(generate_parliamentary_report),
        )
        .route(
            "/api/v1/governance/reporting/parliamentary-reports/{report_id}",
            get(get_parliamentary_report),
        )
        .route(
            "/api/v1/governance/reporting/oag-submissions",
            get(list_oag_submissions).post(submit_to_oag),
        )
        .with_state(state)
}
