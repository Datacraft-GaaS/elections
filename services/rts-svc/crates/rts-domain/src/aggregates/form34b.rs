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

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::aggregates::form34a::CandidateResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Form34BStatus { Pending, Compiled, Verified }
impl Default for Form34BStatus { fn default() -> Self { Self::Pending } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34B {
    pub stream_id: Uuid, pub version: i64,
    pub constituency_code: String, pub county_code: String,
    pub compiled_stations: Vec<String>,
    pub total_registered: u32, pub total_valid: u32, pub total_rejected: u32, pub total_spoilt: u32,
    pub candidate_totals: Vec<CandidateResult>,
    pub status: Form34BStatus,
}
impl Form34B {
    pub fn new(stream_id: Uuid) -> Self {
        Self { stream_id, version: 0, constituency_code: String::new(), county_code: String::new(),
               compiled_stations: Vec::new(), total_registered: 0, total_valid: 0,
               total_rejected: 0, total_spoilt: 0, candidate_totals: Vec::new(), status: Form34BStatus::Pending }
    }
}
impl Aggregate for Form34B {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        if event.event_type == crate::events::EVENT_FORM34B_COMPILED {
            if let Some(v) = event.payload.get("constituency_code").and_then(|v| v.as_str()) { self.constituency_code = v.to_string(); }
            if let Some(v) = event.payload.get("county_code").and_then(|v| v.as_str()) { self.county_code = v.to_string(); }
            if let Some(s) = event.payload.get("compiled_stations") { if let Ok(v) = serde_json::from_value(s.clone()) { self.compiled_stations = v; } }
            if let Some(v) = event.payload.get("total_registered").and_then(|v| v.as_u64()) { self.total_registered = v as u32; }
            if let Some(v) = event.payload.get("total_valid").and_then(|v| v.as_u64()) { self.total_valid = v as u32; }
            if let Some(v) = event.payload.get("total_rejected").and_then(|v| v.as_u64()) { self.total_rejected = v as u32; }
            if let Some(v) = event.payload.get("total_spoilt").and_then(|v| v.as_u64()) { self.total_spoilt = v as u32; }
            if let Some(t) = event.payload.get("candidate_totals") { if let Ok(v) = serde_json::from_value(t.clone()) { self.candidate_totals = v; } }
            self.status = Form34BStatus::Compiled;
        }
        Ok(())
    }
    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot { stream_id: self.stream_id, stream_version: self.version,
                            state: serde_json::to_value(self).unwrap_or_default(), taken_at: Utc::now() }
    }
    fn stream_id(&self) -> Uuid { self.stream_id }
}
