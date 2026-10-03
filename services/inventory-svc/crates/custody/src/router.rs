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

//! HTTP router for the public custody contract (custody/openapi.yaml).

use std::sync::Arc;

use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::state::AppState;

/// router.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/v1/custody/assets",
            get(crate::handlers::list_assets).post(crate::handlers::register_asset),
        )
        .route(
            "/api/v1/custody/assets/{id}/custody",
            patch(crate::handlers::transfer_custody),
        )
        .route(
            "/api/v1/custody/assets/{id}/history",
            get(crate::handlers::get_asset_history),
        )
        .route("/api/v1/custody/seals", post(crate::handlers::create_seal))
        .route(
            "/api/v1/custody/seals/{id}/break",
            patch(crate::handlers::break_seal),
        )
        .route(
            "/api/v1/custody/reserves",
            get(crate::handlers::list_reserves),
        )
        .route(
            "/api/v1/custody/summary",
            get(crate::handlers::get_custody_summary),
        )
        .route(
            "/api/v1/custody/reconcile",
            post(crate::handlers::start_reconciliation),
        )
        .route(
            "/api/v1/custody/reconcile/{id}/report",
            get(crate::handlers::get_reconcile_report),
        )
        .with_state(state)
}
