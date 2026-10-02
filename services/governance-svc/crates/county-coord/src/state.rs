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
