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

//! In-memory projection for the ballot-serial API.
//!
//! The API sub-router previously held no state at all, and the one place it
//! claimed to do arithmetic was the most serious: `trigger_reconciliation` built
//! its own `StationCounts` with `serial_low: 1` and
//! `serial_high: deposited + spoilt`, then fed that to
//! [`crate::reconciliation::reconcile`].
//!
//! That range is constructed so that `assigned == accounted` by definition, so
//! the arithmetic can **never** report a mismatch — which means the hold branch
//! (`ReconciliationStatus::Held`, FR-M63-024 / EC-DAT-11) was unreachable through
//! the API even though the function has its own tests proving it fires. A station
//! could deposit more ballots than it was issued and be cleared.
//!
//! This store holds the real `BallotSerial` aggregate per station, so the
//! reconciliation is fed the range the station was actually assigned.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::aggregate::{BallotSerial, BallotSerialStatus};
use eel_eventstore_rs::aggregate::{Aggregate, StoredEvent};
use uuid::Uuid;

/// One station's ballot-serial record and the events that produced it.
#[derive(Clone)]
pub struct StationRecord {
    /// The folded aggregate.
    pub serial: BallotSerial,
    /// Every event folded in, in order.
    pub events: Vec<StoredEvent>,
    /// Whether a clerk receipt has been confirmed against this assignment.
    pub receipt_confirmed: bool,
    /// The most recent reconciliation result, if one has run.
    pub last_reconciliation: Option<crate::reconciliation::ReconciliationResult>,
}

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    stations: Mutex<HashMap<String, StationRecord>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }

    /// Fold an event into a station's aggregate, returning the new version.
    pub fn apply(
        &self,
        station_code: &str,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<i64, String> {
        let mut guard = self.stations.lock().expect("lock poisoned");
        let record = guard
            .get_mut(station_code)
            .ok_or_else(|| format!("no ballot-serial assignment for station {station_code}"))?;
        let version = record.serial.version + 1;
        let event = StoredEvent {
            stream_id: record.serial.stream_id,
            stream_version: version,
            id: Uuid::now_v7(),
            event_type: event_type.to_string(),
            payload,
            prev_hash: record.events.last().map(|e| e.id.to_string()),
            hlc: chrono::Utc::now().to_rfc3339(),
            recorded_at: chrono::Utc::now(),
        };
        record
            .serial
            .apply(&event)
            .map_err(|e| format!("aggregate rejected the event: {e}"))?;
        record.events.push(event);
        Ok(version)
    }

    /// Create a station record so events can be folded into it.
    pub fn register(&self, station_code: &str) {
        self.stations
            .lock()
            .expect("lock poisoned")
            .entry(station_code.to_string())
            .or_insert_with(|| StationRecord {
                serial: BallotSerial::new(Uuid::now_v7()),
                events: Vec::new(),
                receipt_confirmed: false,
                last_reconciliation: None,
            });
    }

    /// A station's current record.
    pub fn get(&self, station_code: &str) -> Option<StationRecord> {
        self.stations
            .lock()
            .expect("lock poisoned")
            .get(station_code)
            .cloned()
    }

    /// Every station, ordered by code.
    pub fn all_sorted(&self) -> Vec<StationRecord> {
        let guard = self.stations.lock().expect("lock poisoned");
        let mut items: Vec<StationRecord> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.serial.station_code.cmp(&b.serial.station_code));
        items
    }

    /// Mark a station's receipt confirmed.
    pub fn confirm_receipt(&self, station_code: &str) {
        if let Some(r) = self
            .stations
            .lock()
            .expect("lock poisoned")
            .get_mut(station_code)
        {
            r.receipt_confirmed = true;
        }
    }

    /// Whether a clerk receipt has been confirmed.
    pub fn receipt_confirmed(&self, station_code: &str) -> bool {
        self.get(station_code)
            .map(|r| r.receipt_confirmed)
            .unwrap_or(false)
    }

    /// Record a reconciliation result against a station.
    pub fn set_reconciliation(
        &self,
        station_code: &str,
        result: crate::reconciliation::ReconciliationResult,
    ) {
        if let Some(r) = self
            .stations
            .lock()
            .expect("lock poisoned")
            .get_mut(station_code)
        {
            r.last_reconciliation = Some(result);
        }
    }

    /// The status a station is in, as it appears on the wire.
    pub fn status_wire(status: &BallotSerialStatus) -> &'static str {
        match status {
            BallotSerialStatus::Unassigned => "unassigned",
            BallotSerialStatus::Assigned => "assigned",
            BallotSerialStatus::Issued => "issued",
            BallotSerialStatus::Reconciling => "reconciling",
            BallotSerialStatus::Reconciled => "reconciled",
            BallotSerialStatus::Held => "held",
        }
    }
}
