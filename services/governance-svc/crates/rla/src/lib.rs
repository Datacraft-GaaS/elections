pub mod engine;
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
