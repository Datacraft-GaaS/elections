//! REST handlers per ADR-0017.

pub mod health;
pub mod leaves;
pub mod proofs;
pub mod spec;
pub mod sth;
pub mod witnesses;

use axum::{routing::get, Router};

use crate::state::ApiState;

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/transparency/health", get(health::get_health))
        .route("/api/v1/transparency/sth/latest", get(sth::get_latest_sth))
        .route(
            "/api/v1/transparency/sth/{tree_size}",
            get(sth::get_sth_by_size),
        )
        .route(
            "/api/v1/transparency/proof/inclusion",
            get(proofs::get_inclusion_proof),
        )
        .route(
            "/api/v1/transparency/proof/consistency",
            get(proofs::get_consistency_proof),
        )
        .route(
            "/api/v1/transparency/witnesses",
            get(witnesses::list_witnesses),
        )
        .route(
            "/api/v1/transparency/witnesses/cosign",
            axum::routing::post(witnesses::post_cosign),
        )
        .route(
            "/api/v1/transparency/witnesses/report",
            axum::routing::get(witnesses::witness_report),
        )
        .route("/api/v1/transparency/domains", get(sth::list_domains))
        .route(
            "/api/v1/transparency/leaves/{index}",
            get(leaves::get_leaf),
        )
        .route(
            "/v1/tlog/receipts/{code}",
            get(proofs::resolve_receipt),
        )
        .route("/api/v1/log/entries", get(spec::get_log_entries))
        .route("/api/v1/log/leaves", axum::routing::post(spec::append_leaf))
}
