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

//! Route table for the CRMS API. Produces a stateful `Router` that is then
//! wrapped by the shared `eel-api-server-rs` builder.

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Json, Router};

use crate::handlers::*;
use crate::state::AppState;

/// Liveness probe returning a minimal `{"status":"ok"}` payload.
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// Compose the full CRMS route table. Call `.with_state(...)` consumers do so
/// here; the returned `Router` carries no state and composes with
/// `ApiServer::into_router`.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        // Candidates
        .route(
            "/api/v1/crms/candidates",
            get(list_candidates).post(create_candidate),
        )
        .route("/api/v1/crms/candidates/{id}", get(get_candidate))
        .route(
            "/api/v1/crms/candidates/{id}/submit",
            post(submit_candidate),
        )
        .route(
            "/api/v1/crms/candidates/{id}/docs-uploaded",
            post(mark_docs_uploaded),
        )
        .route(
            "/api/v1/crms/candidates/{id}/ro-review",
            post(begin_ro_review),
        )
        .route("/api/v1/crms/candidates/{id}/clear", post(clear_candidate))
        .route(
            "/api/v1/crms/candidates/{id}/reject",
            post(reject_candidate),
        )
        .route(
            "/api/v1/crms/candidates/{id}/gazette",
            post(gazette_candidate),
        )
        .route(
            "/api/v1/crms/candidates/{id}/withdraw",
            post(withdraw_candidate),
        )
        // Nominations
        .route(
            "/api/v1/crms/nominations",
            post(create_nomination),
        )
        .route(
            "/api/v1/crms/nominations/{id}",
            get(get_nomination),
        )
        .route(
            "/api/v1/crms/nominations/{id}/documents",
            post(add_document),
        )
        // Clearance
        .route(
            "/api/v1/crms/clearances",
            get(list_clearances).post(create_clearance),
        )
        .route(
            "/api/v1/crms/clearances/advisory",
            post(advisory_clearance),
        )
        // Quota
        .route(
            "/api/v1/crms/quota/evaluate",
            post(evaluate_quota_rule),
        )
        // Gazette
        .route(
            "/api/v1/crms/gazettes",
            post(build_gazette),
        )
        .route("/api/v1/crms/gazettes/{id}", get(get_gazette))
        .route(
            "/api/v1/crms/gazettes/{id}/publish",
            post(publish_gazette),
        )
        .route("/api/v1/crms/gazettes/{id}/csv", get(gazette_csv))
        .route(
            "/api/v1/crms/gazettes/{id}/json",
            get(gazette_signed_json),
        )
        // Ballot proofs
        .route("/api/v1/crms/ballots", post(build_ballot))
        .route("/api/v1/crms/ballots/{id}", get(get_ballot))
        .route(
            "/api/v1/crms/ballots/{id}/party-sign-off",
            post(ballot_party_sign_off),
        )
        .route(
            "/api/v1/crms/ballots/{id}/approve",
            post(approve_ballot),
        )
        // Parties
        .route(
            "/api/v1/crms/parties",
            get(list_parties).post(create_party),
        )
        .route("/api/v1/crms/parties/{id}", get(get_party))
        .route("/api/v1/crms/parties/{id}/submit", post(submit_party))
        .route("/api/v1/crms/parties/{id}/approve", post(approve_party))
        .route(
            "/api/v1/crms/parties/{id}/suspend",
            post(suspend_party),
        )
        .route(
            "/api/v1/crms/parties/{id}/deregister",
            post(deregister_party),
        )
        // Coalitions
        .route("/api/v1/crms/coalitions", post(create_coalition))
        .route(
            "/api/v1/crms/coalitions/{id}/confirm-member",
            post(confirm_member),
        )
        .route(
            "/api/v1/crms/coalitions/{id}/dissolve",
            post(dissolve_coalition),
        )
        // Primaries
        .route(
            "/api/v1/crms/primaries/results",
            post(submit_primary_result),
        )
        .route(
            "/api/v1/crms/primaries/results/{id}",
            get(get_primary_result),
        )
        .route(
            "/api/v1/crms/primaries/results/{id}/publish",
            post(publish_primary_result),
        )
        .route(
            "/api/v1/crms/primaries/disputes",
            get(list_primary_disputes).post(create_primary_dispute),
        )
        // Payments / fee schedules
        .route(
            "/api/v1/crms/fee-schedules",
            get(list_fee_schedules).post(create_fee_schedule),
        )
        .route("/api/v1/crms/payments", post(record_payment))
        .route(
            "/api/v1/crms/payments/{id}/confirm",
            post(confirm_payment),
        )
        // Party symbols
        .route(
            "/api/v1/crms/symbols",
            get(list_symbols).post(register_symbol),
        )
        .route("/api/v1/crms/symbols/{id}/approve", post(approve_symbol))
        .route("/api/v1/crms/symbols/{id}/reject", post(reject_symbol))
        // Domain events
        .route("/api/v1/crms/events", get(list_events))
        // Public contract endpoints (crms/nomination.yaml quota + ballot-proofs)
        .route(
            "/api/v1/crms/ballot-proofs",
            post(spec::publish_ballot_proof),
        )
        .route(
            "/api/v1/crms/ballot-proofs/{proof_id}",
            get(spec::get_ballot_proof),
        )
        .route(
            "/api/v1/crms/quotas/{scope}/{code}",
            get(spec::get_quota_compliance),
        )
        .merge(public::router())
        .with_state(state)
}
