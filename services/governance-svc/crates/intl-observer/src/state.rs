//! In-memory store for the public intl-observer contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{ObserverMission, ObserverReport};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Missions by `mission_id`.
    pub missions: Mutex<HashMap<String, ObserverMission>>,
    /// Reports by `mission_id`, in insertion order.
    pub reports: Mutex<HashMap<String, Vec<ObserverReport>>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
