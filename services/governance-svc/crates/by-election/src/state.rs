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

//! In-memory store for the public by-election contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eel_memstore_rs::Collection;

use crate::models::{ByElection, CostReport, KitPlan, RegisterSubset};
use crate::spec;

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// By-elections by `by_election_id`, as the legacy public-contract view.
    pub elections: Mutex<HashMap<String, spec::ByElection>>,
    /// By-elections by id, with the typed status the state machine needs.
    ///
    /// Separate from `elections` because that one holds `spec::ByElection`,
    /// whose `status` is a String, while `models::ByElection::status` is the
    /// `ByElectionStatus` enum. Two representations existed and neither was ever
    /// written to: create_by_election built a models::ByElection and dropped it,
    /// while transition_by_election_status mutated a spec::ByElection that
    /// nothing had ever inserted.
    pub by_elections: Collection<ByElection>,
    /// Kit plans by plan id, so a plan can be read back rather than re-minted.
    pub kit_plans: Collection<KitPlan>,
    /// Register subsets by subset id.
    pub register_subsets: Collection<RegisterSubset>,
    /// Cost reports by report id. Only ever populated by submit_cost_report,
    /// which runs the real KES arithmetic in `crate::cost_report`.
    pub cost_reports: Collection<CostReport>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
