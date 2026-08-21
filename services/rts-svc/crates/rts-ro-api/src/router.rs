use crate::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        // M09: pending 34As awaiting RO review
        .route("/api/v1/rts/ro/queue", get(get_queue))
        // M09: RO countersigns after human review
        .route("/api/v1/rts/ro/form34a/{id}/verify", post(verify_form34a))
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
}
