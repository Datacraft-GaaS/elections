//! In-memory store for the public legal-audit contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{TransmissionCertification, TransmissionLegalBasis};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Transmission legal bases by `form_id`.
    pub transmissions: Mutex<HashMap<String, TransmissionLegalBasis>>,
    /// Certifications by `certification_id`.
    pub certifications: Mutex<HashMap<String, TransmissionCertification>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
