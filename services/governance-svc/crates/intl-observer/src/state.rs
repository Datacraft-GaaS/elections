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

//! In-memory store for the intl-observer contract.
//!
//! Two surfaces share this state. The contract-bound
//! `/api/v1/governance/intl-observer/*` endpoints manage `spec` types. The
//! `/v1/observer-missions/*` endpoints manage the legacy `models` types: a
//! mission is registered, briefed, and only an **active** mission may file an
//! observation report, per `ObserverEngine::can_submit_report`.
//!
//! Previously the `/v1/*` endpoints held no state at all: every list returned
//! `vec![]`, `register_mission` hardcoded `workspace_id: "ws-stub"` while
//! discarding the mission, and `submit_report` accepted a report from any mission
//! including one that had never been registered.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{Briefing, JointStatement, ObservationReport, ObserverMission};
use crate::spec::{ObserverMission as SpecObserverMission, ObserverReport};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Contract-bound missions by `mission_id`.
    pub missions: Mutex<HashMap<String, SpecObserverMission>>,
    /// Contract-bound reports by `mission_id`, in insertion order.
    pub reports: Mutex<HashMap<String, Vec<ObserverReport>>>,
    /// Legacy missions by `mission_id`.
    pub legacy_missions: Mutex<HashMap<String, ObserverMission>>,
    /// Legacy briefings by `mission_id`, in insertion order.
    pub briefings: Mutex<HashMap<String, Vec<Briefing>>>,
    /// Legacy observation reports by `mission_id`, in insertion order.
    pub observation_reports: Mutex<HashMap<String, Vec<ObservationReport>>>,
    /// Legacy joint statements by `statement_id`.
    pub joint_statements: Mutex<HashMap<String, JointStatement>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }

    /// All legacy missions, newest first.
    pub fn legacy_missions_sorted(&self) -> Vec<ObserverMission> {
        let guard = self.legacy_missions.lock().expect("lock poisoned");
        let mut items: Vec<ObserverMission> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| b.registered_at.cmp(&a.registered_at).then(b.id.cmp(&a.id)));
        items
    }

    /// Briefings for a mission, oldest first.
    pub fn briefings_for(&self, mission_id: &str) -> Vec<Briefing> {
        let guard = self.briefings.lock().expect("lock poisoned");
        let mut items: Vec<Briefing> = guard.get(mission_id).cloned().unwrap_or_default();
        drop(guard);
        items.sort_by(|a, b| a.scheduled_at.cmp(&b.scheduled_at).then(a.id.cmp(&b.id)));
        items
    }

    /// Observation reports for a mission, oldest first.
    pub fn observation_reports_for(&self, mission_id: &str) -> Vec<ObservationReport> {
        let guard = self.observation_reports.lock().expect("lock poisoned");
        let mut items: Vec<ObservationReport> = guard.get(mission_id).cloned().unwrap_or_default();
        drop(guard);
        items.sort_by(|a, b| a.submitted_at.cmp(&b.submitted_at).then(a.id.cmp(&b.id)));
        items
    }

    /// All joint statements, newest first.
    pub fn joint_statements_sorted(&self) -> Vec<JointStatement> {
        let guard = self.joint_statements.lock().expect("lock poisoned");
        let mut items: Vec<JointStatement> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| b.published_at.cmp(&a.published_at).then(b.id.cmp(&a.id)));
        items
    }
}
