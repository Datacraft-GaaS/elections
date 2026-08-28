//! BVI backend REST API (modules M05 bvi-device, M06 bvi-backend).
//!
//! Contract: `packages/openapi-contracts/bvi/openapi.yaml` v1.0.0.
//! Conventions per ADR-0011: Idempotency-Key on all writes with a 24 hour
//! response snapshot replay, RFC 9457 problem+json errors, x-eel-tier aware
//! auth posture (internal mTLS terminated at the Envoy sidecar, tier 2/3 JWT).

pub mod error;
pub mod idem;
pub mod models;
pub mod routes;
pub mod state;
pub mod store;

use axum::Router;

/// Build the full API router for any store implementation.
pub fn app<S: store::BviStore>(state: state::AppState<S>) -> Router {
    routes::router(state)
}
