//! In-memory store for the public forensics contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{EvidenceAttachment, Finding, Investigation};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Investigations by `investigation_id`.
    pub investigations: Mutex<HashMap<String, Investigation>>,
    /// Evidence attachments by `investigation_id`, in insertion order.
    pub evidence: Mutex<HashMap<String, Vec<EvidenceAttachment>>>,
    /// Findings by `investigation_id`, in insertion order.
    pub findings: Mutex<HashMap<String, Vec<Finding>>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
