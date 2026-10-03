// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! In-memory store for the forensics contract.
//!
//! Two surfaces share this state. The contract-bound
//! `/api/v1/governance/forensics/*` endpoints manage investigations, evidence and
//! findings. The `/v1/forensics/*` endpoints manage analysis cycles: a cycle is
//! created, run, and only a complete cycle may publish a replication package or a
//! public report. Every one of those transitions is a state change against a
//! stored record -- previously the endpoints answered from `vec![]` or minted a
//! response the cycle store never saw, so a cycle could be "run" without existing
//! and a replication package could be published against a draft.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{
    AnalysisCycle, EstimateRecord, PublicReport, ReferralPacket, ReplicationPackage,
};
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
    /// Analysis cycles by `cycle_id`.
    pub cycles: Mutex<HashMap<String, AnalysisCycle>>,
    /// Estimates by `estimate_id`.
    pub estimates: Mutex<HashMap<String, EstimateRecord>>,
    /// Referral packets by `referral_id`.
    pub referrals: Mutex<HashMap<String, ReferralPacket>>,
    /// Replication packages by `package_id`.
    pub replication: Mutex<HashMap<String, ReplicationPackage>>,
    /// Published public reports by `report_id`.
    pub reports: Mutex<HashMap<String, PublicReport>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }

    /// All cycles, newest first.
    pub fn cycles_sorted(&self) -> Vec<AnalysisCycle> {
        let guard = self.cycles.lock().expect("lock poisoned");
        let mut items: Vec<AnalysisCycle> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
        items
    }

    /// All estimates for a cycle, oldest first.
    pub fn estimates_for(&self, cycle_id: &str) -> Vec<EstimateRecord> {
        let guard = self.estimates.lock().expect("lock poisoned");
        let mut items: Vec<EstimateRecord> = guard
            .values()
            .filter(|e| e.cycle_id == cycle_id)
            .cloned()
            .collect();
        drop(guard);
        items.sort_by(|a, b| a.id.cmp(&b.id));
        items
    }

    /// All public reports, newest first.
    pub fn reports_sorted(&self) -> Vec<PublicReport> {
        let guard = self.reports.lock().expect("lock poisoned");
        let mut items: Vec<PublicReport> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| b.published_at.cmp(&a.published_at).then(b.id.cmp(&a.id)));
        items
    }
}
