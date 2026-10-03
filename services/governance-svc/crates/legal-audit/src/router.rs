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
            "/legal-audit/transmissions",
            get(list_transmissions).post(record_transmission),
        )
        .route(
            "/legal-audit/transmissions/{record_id}/verdict",
            get(get_verdict),
        )
        .route("/legal-audit/verdicts", get(list_verdicts))
        .route(
            "/legal-audit/parties/{party_id}/summary",
            get(party_summary),
        )
        .route("/legal-audit/reports", post(generate_report))
        // Public contract endpoints (governance-legal-audit/audit.yaml)
        .route(
            "/api/v1/governance/legal-audit/transmissions",
            get(list_transmission_legal_records),
        )
        .route(
            "/api/v1/governance/legal-audit/transmissions/{form_id}",
            get(get_transmission_legal_basis),
        )
        .route(
            "/api/v1/governance/legal-audit/certifications",
            post(certify_transmission),
        )
        .route(
            "/api/v1/governance/legal-audit/reports/{report_id}",
            get(get_legal_audit_report),
        )
        .with_state(state)
}
