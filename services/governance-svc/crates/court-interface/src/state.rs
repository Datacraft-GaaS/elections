//! In-memory store for the public court-interface contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{EvidenceRequest, PetitionRecord};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Petitions by `petition_id`.
    pub petitions: Mutex<HashMap<String, PetitionRecord>>,
    /// Evidence requests by `petition_id`, in insertion order.
    pub evidence_requests: Mutex<HashMap<String, Vec<EvidenceRequest>>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
