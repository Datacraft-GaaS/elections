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
    routing::{get, patch, post},
    Router,
};

use crate::state::AppState;

pub mod diagnostics;
pub mod spec;
pub mod tickets;
pub mod warranty;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/healthz",
            get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }),
        )
        .route(
            "/v1/support/tickets",
            get(tickets::list_tickets).post(tickets::create_ticket),
        )
        .route("/v1/support/tickets/{id}", get(tickets::get_ticket))
        .route(
            "/v1/support/tickets/{id}/diagnostics",
            post(diagnostics::run_diagnostic),
        )
        .route("/v1/support/warranty", post(warranty::submit_claim))
        // Public contract endpoints (field-mobile-support/support.yaml)
        .route(
            "/api/v1/mobile/support/tickets",
            get(spec::list_support_tickets).post(spec::create_support_ticket),
        )
        .route(
            "/api/v1/mobile/support/tickets/{ticket_id}",
            get(spec::get_support_ticket).patch(spec::update_support_ticket),
        )
        .route(
            "/api/v1/mobile/support/knowledge-base",
            get(spec::list_knowledge_base_articles),
        )
        .route(
            "/api/v1/mobile/support/knowledge-base/{article_id}",
            get(spec::get_knowledge_base_article),
        )
        .with_state(state)
}
