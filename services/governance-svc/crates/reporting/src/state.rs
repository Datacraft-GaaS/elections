//! In-memory store for the public reporting contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{OagSubmission, ReportJob};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Report jobs by `job_id`.
    pub reports: Mutex<HashMap<String, ReportJob>>,
    /// OAG submissions by `submission_id`.
    pub oag_submissions: Mutex<HashMap<String, OagSubmission>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
