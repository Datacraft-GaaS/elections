use std::sync::Arc;

use axum::{
    routing::{get, post, patch},
    Router,
};

use crate::handlers::{
    binary_integrity_check, file_grievance, get_anomalies, get_queue_snapshot, get_results,
    get_results_by_scope, get_token_proof, health, report_clone, sse_events, verify_token,
};
use crate::spec::{BffState};
use crate::sse::ShardedBroadcast;

pub fn build_router() -> Router {
    let broadcast = ShardedBroadcast::new();
    build_router_with_state(broadcast)
}

pub fn build_router_with_test_state(
    state: Arc<BffState>, 
    broadcast: Arc<ShardedBroadcast>
) -> Router {
    // Delegate to spec::router which has all the BFF routes.
    // The legacy /verify/v1/* and /health routes are also merged.
    Router::new()
        .route("/health", get(health))
        .route("/verify/v1/results", get(get_results))
        .route(
            "/verify/v1/results/{scope}/{code}",
            get(get_results_by_scope),
        )
        .route("/verify/v1/token", post(verify_token))
        .route("/verify/v1/token/{sha}/proof", get(get_token_proof))
        .route("/verify/v1/grievances", post(file_grievance))
        .route("/verify/v1/events", get(sse_events))
        .route("/verify/v1/queue/{station}", get(get_queue_snapshot))
        .route("/verify/v1/anomalies", get(get_anomalies))
        .route("/verify/v1/clone-report", post(report_clone))
        .route("/verify/v1/binary-integrity", post(binary_integrity_check))
        .merge(crate::spec::router(state))
        .with_state(broadcast)
}

pub fn build_router_with_state(broadcast: Arc<ShardedBroadcast>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/verify/v1/results", get(get_results))
        .route(
            "/verify/v1/results/{scope}/{code}",
            get(get_results_by_scope),
        )
        .route("/verify/v1/token", post(verify_token))
        .route("/verify/v1/token/{sha}/proof", get(get_token_proof))
        .route("/verify/v1/grievances", post(file_grievance))
        .route("/verify/v1/events", get(sse_events))
        .route("/verify/v1/queue/{station}", get(get_queue_snapshot))
        .route("/verify/v1/anomalies", get(get_anomalies))
        .route("/verify/v1/clone-report", post(report_clone))
        .route("/verify/v1/binary-integrity", post(binary_integrity_check))
        .merge(crate::spec::router(Arc::new(crate::spec::BffState::new())))
        .with_state(broadcast)
}
