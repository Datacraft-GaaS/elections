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

//! In-memory store for the public court-interface contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eel_memstore_rs::Collection;

use crate::models::{SupplementaryRequest, TechnicalWitness};
use crate::spec::{EvidenceRequest, PetitionRecord};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Petitions by `petition_id`.
    pub petitions: Mutex<HashMap<String, PetitionRecord>>,
    /// Evidence requests by `petition_id`, in insertion order.
    pub evidence_requests: Mutex<HashMap<String, Vec<EvidenceRequest>>>,
    /// Technical witnesses by `witness_id`.
    pub witnesses: Collection<TechnicalWitness>,
    /// Supplementary requests by `request_id`.
    pub supplementary: Collection<SupplementaryRequest>,
}

impl AppState {
    /// Whether a petition exists. Used to 404 rather than fabricate a record for
    /// a petition id that was never filed.
    pub fn has_petition(&self, petition_id: &str) -> bool {
        self.petitions
            .lock()
            .expect("court petitions lock poisoned")
            .contains_key(petition_id)
    }

    /// Evidence requests recorded against a petition.
    pub fn requests_for(&self, petition_id: &str) -> Vec<EvidenceRequest> {
        self.evidence_requests
            .lock()
            .expect("court evidence requests lock poisoned")
            .get(petition_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }
}
