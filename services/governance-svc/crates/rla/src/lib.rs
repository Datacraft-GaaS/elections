//! Risk-limiting audit — BRAVO audit lifecycle over [`Store`].
#![warn(missing_docs)]
pub mod engine;
/// HTTP handlers driving the BRAVO audit lifecycle over [`Store`].
pub mod handlers;
pub mod manifest;
pub mod models;
pub mod router;
pub mod spec;
pub mod spec_state;
pub mod store;

pub use spec_state::{SharedSpecState, SpecState};

/// Composite router state: the internal BRAVO store plus the public spec
/// tracker. `FromRef` lets handlers extract either piece independently.
#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub store: store::Store,
    pub spec_state: SharedSpecState,
}
