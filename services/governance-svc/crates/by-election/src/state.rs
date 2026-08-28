//! In-memory store for the public by-election contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::ByElection;

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// By-elections by `by_election_id`.
    pub elections: Mutex<HashMap<String, ByElection>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
