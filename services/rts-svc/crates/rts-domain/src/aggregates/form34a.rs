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
use crate::candidate_status::CandidateStatusAnnotation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateResult {
    pub candidate_id: String,
    pub candidate_name: String,
    pub votes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateAnnotation {
    pub candidate_id: String,
    pub annotation: CandidateStatusAnnotation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Form34AStatus { Draft, Captured, Verified, Amended, Rejected }

impl Default for Form34AStatus { fn default() -> Self { Self::Draft } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34A {
    pub stream_id: Uuid,
    pub version: i64,
    pub station_code: String,
    pub constituency_code: String,
    pub county_code: String,
    pub registered_voters: u32,
    pub votes_cast: u32,
    pub valid_votes: u32,
    pub rejected_votes: u32,
    pub spoilt_ballots: u32,
    pub candidates: Vec<CandidateResult>,
    pub status: Form34AStatus,
    pub raw_scan_ref: Option<String>,
    pub candidate_annotations: Vec<CandidateAnnotation>,
}

impl Form34A {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id, version: 0,
            station_code: String::new(), constituency_code: String::new(), county_code: String::new(),
            registered_voters: 0, votes_cast: 0, valid_votes: 0, rejected_votes: 0, spoilt_ballots: 0,
            candidates: Vec::new(), status: Form34AStatus::Draft,
            raw_scan_ref: None, candidate_annotations: Vec::new(),
        }
    }
}

impl Aggregate for Form34A {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        match event.event_type.as_str() {
            crate::events::EVENT_FORM34A_CAPTURED => {
                if let Some(v) = event.payload.get("station_code").and_then(|v| v.as_str()) { self.station_code = v.to_string(); }
                if let Some(v) = event.payload.get("constituency_code").and_then(|v| v.as_str()) { self.constituency_code = v.to_string(); }
                if let Some(v) = event.payload.get("county_code").and_then(|v| v.as_str()) { self.county_code = v.to_string(); }
                if let Some(v) = event.payload.get("registered_voters").and_then(|v| v.as_u64()) { self.registered_voters = v as u32; }
                if let Some(v) = event.payload.get("votes_cast").and_then(|v| v.as_u64()) { self.votes_cast = v as u32; }
                if let Some(v) = event.payload.get("valid_votes").and_then(|v| v.as_u64()) { self.valid_votes = v as u32; }
                if let Some(v) = event.payload.get("rejected_votes").and_then(|v| v.as_u64()) { self.rejected_votes = v as u32; }
                if let Some(v) = event.payload.get("spoilt_ballots").and_then(|v| v.as_u64()) { self.spoilt_ballots = v as u32; }
                if let Some(c) = event.payload.get("candidates") {
                    if let Ok(v) = serde_json::from_value::<Vec<CandidateResult>>(c.clone()) { self.candidates = v; }
                }
                if let Some(v) = event.payload.get("raw_scan_ref").and_then(|v| v.as_str()) { self.raw_scan_ref = Some(v.to_string()); }
                self.status = Form34AStatus::Captured;
            }
            crate::events::EVENT_FORM34A_VERIFIED => { self.status = Form34AStatus::Verified; }
            crate::events::EVENT_FORM34A_AMENDED => {
                if let Some(v) = event.payload.get("valid_votes").and_then(|v| v.as_u64()) { self.valid_votes = v as u32; }
                if let Some(v) = event.payload.get("rejected_votes").and_then(|v| v.as_u64()) { self.rejected_votes = v as u32; }
                if let Some(v) = event.payload.get("spoilt_ballots").and_then(|v| v.as_u64()) { self.spoilt_ballots = v as u32; }
                if let Some(v) = event.payload.get("votes_cast").and_then(|v| v.as_u64()) { self.votes_cast = v as u32; }
                if let Some(c) = event.payload.get("candidates") {
                    if let Ok(v) = serde_json::from_value::<Vec<CandidateResult>>(c.clone()) { self.candidates = v; }
                }
                self.status = Form34AStatus::Amended;
            }
            crate::events::EVENT_FORM34A_REJECTED => { self.status = Form34AStatus::Rejected; }
            _ => {}
        }
        Ok(())
    }
    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot {
            stream_id: self.stream_id, stream_version: self.version,
            state: serde_json::to_value(self).unwrap_or_default(), taken_at: Utc::now(),
        }
    }
    fn stream_id(&self) -> Uuid { self.stream_id }
}
