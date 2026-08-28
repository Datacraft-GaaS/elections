//! Shared handler state.

/// Application state threaded into every axum handler. Generic over the
/// store so tests can run the full router against an in-memory fake.
#[derive(Clone)]
pub struct AppState<S> {
    pub store: S,
}

impl<S: crate::store::BviStore> AppState<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
