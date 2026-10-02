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

use axum::{
	routing::{get, post},
	Router,
};

use crate::state::AppState;

pub mod assignments;
pub mod checkin;
pub mod checkout;
pub mod ping;
pub mod spec;
pub mod status;

pub fn router(state: AppState) -> Router {
	Router::new()
		.route(
			"/healthz",
			get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }),
		)
		.route(
			"/v1/dispatch/assignments/{worker_id}",
			get(assignments::get_assignments),
		)
		.route("/v1/dispatch/check-in", post(checkin::check_in))
		.route("/v1/dispatch/ping", post(ping::status_ping))
		.route("/v1/dispatch/check-out", post(checkout::check_out))
		.route("/v1/dispatch/workers/{id}/status", get(status::get_status))
		// Public contract endpoints (field-mobile-dispatch/dispatch.yaml)
		.route("/api/v1/mobile/dispatch/assignments", get(spec::list_dispatch_assignments).post(spec::create_dispatch_assignment))
		.route("/api/v1/mobile/dispatch/assignments/{assignment_id}", get(spec::get_dispatch_assignment).patch(spec::update_dispatch_assignment))
		.route("/api/v1/mobile/dispatch/bulk-assignments", post(spec::create_bulk_assignments))
		.route("/api/v1/mobile/dispatch/coverage/{constituency_code}", get(spec::get_constituency_coverage))
		.with_state(state)
}
