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

//! BallotSerial event-sourced aggregate (WP-3.3, M63).

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::*;

// ---- BallotSerial -----------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BallotSerialStatus {
    Unassigned,
    Assigned,
    Issued,
    /// Reconciliation in progress.
    Reconciling,
    Reconciled,
    /// Hold raised; blocks M10 result publication (FR-M63-025).
    Held,
}

impl Default for BallotSerialStatus {
    fn default() -> Self {
        Self::Unassigned
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotSerial {
    pub stream_id: Uuid,
    pub version: i64,
    pub station_code: String,
    pub lot_id: String,
    pub serial_low: u64,
    pub serial_high: u64,
    pub status: BallotSerialStatus,
    pub spoilt_serials: Vec<u64>,
    /// Whether this station has an active result hold (never silently cleared).
    pub result_hold: bool,
    pub hold_reason: Option<String>,
    pub assigned_by: String,
    pub assigned_at: Option<String>,
}

impl BallotSerial {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            station_code: String::new(),
            lot_id: String::new(),
            serial_low: 0,
            serial_high: 0,
            status: BallotSerialStatus::Unassigned,
            spoilt_serials: Vec::new(),
            result_hold: false,
            hold_reason: None,
            assigned_by: String::new(),
            assigned_at: None,
        }
    }

    /// Validate that a serial falls within this station's assigned range (FR-M63-014).
    pub fn in_range(&self, serial: u64) -> bool {
        serial >= self.serial_low && serial <= self.serial_high
    }

    pub fn assigned_count(&self) -> u64 {
        if self.serial_high >= self.serial_low {
            self.serial_high - self.serial_low + 1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    fn assigned() -> BallotSerial {
        let mut s = BallotSerial::new(Uuid::new_v4());
        s.serial_low = 100;
        s.serial_high = 200;
        s
    }

    #[trace("FR-M63-014")]
    #[test]
    fn in_range_accepts_boundaries_and_rejects_outside() {
        let s = assigned();
        assert!(s.in_range(100));
        assert!(s.in_range(200));
        assert!(s.in_range(150));
        assert!(!s.in_range(99));
        assert!(!s.in_range(201));
    }

    #[trace("FR-M63-014")]
    #[test]
    fn assigned_count_is_inclusive() {
        assert_eq!(assigned().assigned_count(), 101);
    }
}

impl Aggregate for BallotSerial {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        match event.event_type.as_str() {
            EVENT_SERIAL_ASSIGNED => {
                self.status = BallotSerialStatus::Assigned;
                if let Some(v) = event.payload.get("station_code").and_then(|v| v.as_str()) {
                    self.station_code = v.to_string();
                }
                if let Some(v) = event.payload.get("lot_id").and_then(|v| v.as_str()) {
                    self.lot_id = v.to_string();
                }
                if let Some(v) = event.payload.get("serial_low").and_then(|v| v.as_u64()) {
                    self.serial_low = v;
                }
                if let Some(v) = event.payload.get("serial_high").and_then(|v| v.as_u64()) {
                    self.serial_high = v;
                }
                if let Some(v) = event.payload.get("assigned_by").and_then(|v| v.as_str()) {
                    self.assigned_by = v.to_string();
                }
                if let Some(v) = event.payload.get("assigned_at").and_then(|v| v.as_str()) {
                    self.assigned_at = Some(v.to_string());
                }
            }
            EVENT_BALLOT_ISSUED => {
                self.status = BallotSerialStatus::Issued;
            }
            EVENT_BALLOT_SPOILT => {
                if let Some(v) = event.payload.get("spoilt_serial").and_then(|v| v.as_u64()) {
                    if !self.spoilt_serials.contains(&v) {
                        self.spoilt_serials.push(v);
                    }
                }
            }
            EVENT_RECONCILIATION_STARTED => {
                self.status = BallotSerialStatus::Reconciling;
            }
            EVENT_RECONCILIATION_COMPLETE => {
                self.status = BallotSerialStatus::Reconciled;
                self.result_hold = false;
            }
            EVENT_RECONCILIATION_DISCREPANCY => {
                self.status = BallotSerialStatus::Held;
                self.result_hold = true;
                if let Some(v) = event.payload.get("flag_reason").and_then(|v| v.as_str()) {
                    self.hold_reason = Some(v.to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot {
            stream_id: self.stream_id,
            stream_version: self.version,
            state: serde_json::to_value(self).unwrap_or_default(),
            taken_at: Utc::now(),
        }
    }

    fn stream_id(&self) -> Uuid {
        self.stream_id
    }
}
