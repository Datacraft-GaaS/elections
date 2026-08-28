//! crms-api — HTTP surface for the Candidate Registration & Management Service.
//!
//! WP-4.11. Exposes the crms-domain aggregates (candidates, nominations,
//! clearance, quota, gazette, ballot, party/coalition, primaries, payments,
//! symbols) as axum endpoints. The router is composed with the shared
//! `eel-api-server-rs` builder for auth, load-shedding, idempotency, and
//! observability plumbing.

#![warn(missing_docs)]

pub mod handlers;
pub mod models;
pub mod router;
pub mod spec;
pub mod state;

pub use router::build_router;
pub use state::AppState;
