//! In-memory store for the reporting contract.
//!
//! Two surfaces share this state. The contract-bound
//! `/api/v1/governance/reporting/*` endpoints manage `spec` report jobs and OAG
//! submissions. The `/reporting/*` endpoints manage `models` statutory reports
//! and the downstream artefacts they gate: a testimony workspace, a judiciary
//! handover and an SC/IT pre-registration.
//!
//! Every one of those five previously answered from `vec![]` or
//! `Default::default()`. The gating rule is the point: a testimony workspace is
//! opened against a **ready** report, and a report is only ready once its period
//! has been generated. `get_report` returned a blank `StatutoryReport` for any
//! id, so a request for a report that had never been generated returned an empty
//! report with `200 OK`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{JudiciaryHandover, ScPreregistration, StatutoryReport, TestimonyWorkspace};
use crate::spec::{OagSubmission, ReportJob};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Contract-bound report jobs by `job_id`.
    pub reports: Mutex<HashMap<String, ReportJob>>,
    /// OAG submissions by `submission_id`.
    pub oag_submissions: Mutex<HashMap<String, OagSubmission>>,
    /// Statutory reports by `report_id`.
    pub statutory_reports: Mutex<HashMap<String, StatutoryReport>>,
    /// Testimony workspaces by `workspace_id`.
    pub workspaces: Mutex<HashMap<String, TestimonyWorkspace>>,
    /// Judiciary handovers by `handover_id`.
    pub handovers: Mutex<HashMap<String, JudiciaryHandover>>,
    /// SC/IT pre-registrations by `prereg_id`.
    pub preregs: Mutex<HashMap<String, ScPreregistration>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }

    /// Statutory reports, newest first.
    pub fn statutory_reports_sorted(&self) -> Vec<StatutoryReport> {
        let guard = self.statutory_reports.lock().expect("lock poisoned");
        let mut items: Vec<StatutoryReport> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| b.generated_at.cmp(&a.generated_at).then(b.report_id.cmp(&a.report_id)));
        items
    }

    /// Testimony workspaces, oldest first.
    pub fn workspaces_sorted(&self) -> Vec<TestimonyWorkspace> {
        let guard = self.workspaces.lock().expect("lock poisoned");
        let mut items: Vec<TestimonyWorkspace> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.workspace_id.cmp(&b.workspace_id)));
        items
    }

    /// Whether a statutory report exists.
    pub fn report_exists(&self, report_id: &str) -> bool {
        self.statutory_reports
            .lock()
            .expect("lock poisoned")
            .contains_key(report_id)
    }

    /// A report's status, if it exists.
    pub fn report_status(&self, report_id: &str) -> Option<crate::models::ReportStatus> {
        self.statutory_reports
            .lock()
            .expect("lock poisoned")
            .get(report_id)
            .map(|r| r.status.clone())
    }
}
