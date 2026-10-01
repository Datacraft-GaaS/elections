//! In-memory store for the public county-coord contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eel_memstore_rs::Collection;

use crate::models::{CoordinationIncident, CountyDataset, Mou};
use crate::spec::{CoordinationEscalation, CountyStatusReport};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Status reports by `report_id`.
    pub reports: Mutex<HashMap<String, CountyStatusReport>>,
    /// Escalations by `escalation_id`.
    pub escalations: Mutex<HashMap<String, CoordinationEscalation>>,
    /// MOUs by `id`. Create, sign and list round-trip through this.
    pub mous: Collection<Mou>,
    /// Submitted datasets by `id`.
    pub datasets: Collection<CountyDataset>,
    /// Coordination incidents by `id`.
    pub incidents: Collection<CoordinationIncident>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
