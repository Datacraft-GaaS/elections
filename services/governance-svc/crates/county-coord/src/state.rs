//! In-memory store for the public county-coord contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{CoordinationEscalation, CountyStatusReport};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Status reports by `report_id`.
    pub reports: Mutex<HashMap<String, CountyStatusReport>>,
    /// Escalations by `escalation_id`.
    pub escalations: Mutex<HashMap<String, CoordinationEscalation>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
