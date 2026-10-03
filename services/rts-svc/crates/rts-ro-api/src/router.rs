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

use crate::commit_reveal;
use crate::handlers::*;
use crate::recording_continuity::{certify_session, Certification};
use axum::{
    routing::{get, post},
    Json, Router,
};

/// The router with an empty HSM trust anchor.
///
/// `verify_form34a` will refuse: the RO desktop HSM public keys are provisioned per
/// officer and are not in this workspace. Use [`build_router_with`] to supply them.
pub fn build_router() -> Router {
    build_router_with(crate::ro_trust::RoTrust::new().shared())
}

/// The router, trusting the given RO HSM keys.
pub fn build_router_with(trust: crate::ro_trust::SharedRoTrust) -> Router {
    // The layer goes on LAST: in axum a `layer` applies only to the routes added
    // before it, so putting it first would leave every route without the trust
    // anchor and `Extension` would reject the request with a 500.
    Router::new()
        .route("/health", get(health))
        // M09: pending 34As awaiting RO review
        .route("/api/v1/rts/ro/queue", get(get_queue))
        // M09: RO countersigns after human review
        .route("/api/v1/rts/form34a/{id}/verify", post(verify_form34a))
        // M09: edit-with-reason; produces amendment event with lineage
        .route("/api/v1/rts/ro/form34a/{id}/amend", post(amend_form34a))
        // M09: constituency-level 34B compilation (RO + witness dual-officer)
        .route("/api/v1/rts/ro/form34b/compile", post(compile_form34b))
        // M09: national-level 34C compilation (CRO role gated)
        .route("/api/v1/rts/ro/form34c/compile", post(compile_form34c))
        // M09: raise dispute case
        .route("/api/v1/rts/ro/disputes", post(create_dispute))
        // M09: party-agent countersignature
        .route("/api/v1/rts/ro/agents/countersign", post(agent_countersign))
        // M09: session recording start/anchor/stop; hash to transparency-svc
        .route("/api/v1/rts/ro/session-recording", post(session_recording))
        // N5: certify recording-anchor continuity (pure; late/batched anchors OK)
        .route(
            "/api/v1/rts/ro/session-recording/certify",
            post(recording_certify),
        )
        // N1: commit-reveal for 34A tallies (idempotent late/out-of-order commits)
        .route(
            "/api/v1/rts/forms/commit",
            post(commit_reveal::forms_commit),
        )
        .route(
            "/api/v1/rts/forms/arrive",
            post(commit_reveal::forms_arrive),
        )
        .route(
            "/api/v1/rts/forms/commitments",
            get(commit_reveal::list_commitments),
        )
        .layer(axum::Extension(trust))
}

// POST /api/v1/rts/ro/session-recording/certify
// Stateless handler over the pure continuity function: accepts whatever
// anchor batches have landed so far, in any order.
async fn recording_certify(
    Json(req): Json<crate::recording_continuity::CertifyRequest>,
) -> Json<Certification> {
    Json(certify_session(
        &req.anchors,
        req.first_seq,
        req.last_seq,
        req.opened,
        req.stopped,
    ))
}
