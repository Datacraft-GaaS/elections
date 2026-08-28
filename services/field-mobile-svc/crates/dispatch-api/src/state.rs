use std::sync::{Arc, Mutex};

use crate::models::{WorkerAssignment, WorkerCheckIn, WorkerCheckOut, WorkerStatusPing};
use crate::spec::DispatchAssignment;

/// In-memory store for the dispatch service (M33).
///
/// Real deployments replace this with a Postgres-backed store; the handlers
/// only depend on the `AppState` surface so the swap is internal.
#[derive(Clone, Default)]
pub struct AppState {
    pub check_ins: Arc<Mutex<Vec<WorkerCheckIn>>>,
    pub check_outs: Arc<Mutex<Vec<WorkerCheckOut>>>,
    pub pings: Arc<Mutex<Vec<WorkerStatusPing>>>,
    pub assignments: Arc<Mutex<Vec<WorkerAssignment>>>,
    pub spec_assignments: Arc<Mutex<std::collections::HashMap<String, DispatchAssignment>>>,
}
