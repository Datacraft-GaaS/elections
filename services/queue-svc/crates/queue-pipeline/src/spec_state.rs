//! In-memory store for the public queue contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::QueueObservation;

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Observations by `station_code`, in insertion order.
    pub observations: Mutex<HashMap<String, Vec<QueueObservation>>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
