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

pub mod applications;
pub mod handoff;
pub mod health;
pub mod otp;
pub mod spec;

use axum::{routing::get, routing::post, Router};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route(
            "/vprs/v1/applications",
            post(applications::create_application),
        )
        .route(
            "/vprs/v1/applications/:id/status",
            get(applications::get_status),
        )
        .route(
            "/vprs/v1/applications/:id",
            axum::routing::delete(applications::cancel_application),
        )
        .route("/vprs/v1/handoff/:token", get(handoff::get_handoff))
        .route("/vprs/v1/otp/request", post(otp::request_otp))
        .route("/vprs/v1/otp/verify", post(otp::verify_otp))
        // Public contract endpoints (vprs/portal.yaml)
        .route(
            "/api/v1/vprs/applications",
            post(applications::create_application),
        )
        .route(
            "/api/v1/vprs/applications/{app_id}",
            get(applications::get_status).delete(applications::cancel_application),
        )
        .route("/api/v1/vprs/handoff/{token}", get(handoff::get_handoff))
        .route("/api/v1/vprs/otp/request", post(otp::request_otp))
        .route("/api/v1/vprs/otp/verify", post(otp::verify_otp))
        .route(
            "/api/v1/vprs/admin/analytics",
            get(spec::get_vprs_analytics),
        )
        .with_state(state)
}
