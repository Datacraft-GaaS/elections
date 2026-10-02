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

//! In-memory store for the public legal-audit contract.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{ComplianceVerdict, TransmissionRecord};
use crate::spec::{TransmissionCertification, TransmissionLegalBasis};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Transmission legal bases by `form_id`.
    pub transmissions: Mutex<HashMap<String, TransmissionLegalBasis>>,
    /// Legacy transmissions by `record_id`.
    pub transmissions_legacy: Mutex<Vec<TransmissionRecord>>,
    /// Verdicts in chain order. Appended to, never rewritten.
    pub verdicts: Mutex<Vec<ComplianceVerdict>>,
    /// Legacy transmissions by `party_id` (the `sender` field), in insertion
    /// order.
    pub by_party: Mutex<HashMap<String, Vec<String>>>,
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

    /// Record a transmission and evaluate it, appending its verdicts to the
    /// chain.
    ///
    /// The whole step holds the chain lock so the new verdicts' predecessor is
    /// the chain head at the moment they were computed. Evaluating outside the
    /// lock would let two concurrent records both chain onto the same head and
    /// fork the sequence.
    pub fn record_and_evaluate(&self, record: TransmissionRecord) -> Vec<ComplianceVerdict> {
        let mut chain = self.verdicts.lock().expect("lock poisoned");
        let produced = crate::rules::evaluate_record(
            &record,
            chain.len(),
            &crate::rules::chain_head(&chain),
            chrono::Utc::now().to_rfc3339(),
        );
        chain.extend(produced.iter().cloned());
        drop(chain);
        self.transmissions_legacy
            .lock()
            .expect("lock poisoned")
            .push(record.clone());
        self.by_party
            .lock()
            .expect("lock poisoned")
            .entry(record.sender.clone())
            .or_default()
            .push(record.record_id.clone());
        produced
    }

    /// Every verdict for a record, in chain order.
    pub fn verdicts_for(&self, record_id: &str) -> Vec<ComplianceVerdict> {
        let guard = self.verdicts.lock().expect("lock poisoned");
        guard.iter().filter(|v| v.record_id == record_id).cloned().collect()
    }

    /// Records sent by a party, in insertion order.
    pub fn records_for_party(&self, party_id: &str) -> Vec<TransmissionRecord> {
        let ids: Vec<String> = self
            .by_party
            .lock()
            .expect("lock poisoned")
            .get(party_id)
            .cloned()
            .unwrap_or_default();
        let guard = self.transmissions_legacy.lock().expect("lock poisoned");
        ids.iter()
            .filter_map(|id| guard.iter().find(|r| &r.record_id == id).cloned())
            .collect()
    }

    /// Whether any transmission names `party_id` as sender or recipient.
    pub fn party_exists(&self, party_id: &str) -> bool {
        if self
            .by_party
            .lock()
            .expect("lock poisoned")
            .contains_key(party_id)
        {
            return true;
        }
        self.transmissions_legacy
            .lock()
            .expect("lock poisoned")
            .iter()
            .any(|r| r.recipient == party_id)
    }
}
