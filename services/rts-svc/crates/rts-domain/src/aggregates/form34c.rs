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

use crate::aggregates::form34a::CandidateResult;
use chrono::{DateTime, Utc};
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Form34CStatus {
    Pending,
    Compiled,
    Published,
}
impl Default for Form34CStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34C {
    pub stream_id: Uuid,
    pub version: i64,
    pub compiled_constituencies: Vec<String>,
    pub total_registered: u64,
    pub total_valid: u64,
    pub total_rejected: u64,
    pub total_spoilt: u64,
    pub candidate_totals: Vec<CandidateResult>,
    pub merkle_root: Option<String>,
    pub status: Form34CStatus,
    pub published_at: Option<DateTime<Utc>>,
}
impl Form34C {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            compiled_constituencies: Vec::new(),
            total_registered: 0,
            total_valid: 0,
            total_rejected: 0,
            total_spoilt: 0,
            candidate_totals: Vec::new(),
            merkle_root: None,
            status: Form34CStatus::Pending,
            published_at: None,
        }
    }
}
impl Aggregate for Form34C {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        match event.event_type.as_str() {
            crate::events::EVENT_FORM34C_COMPILED => {
                if let Some(c) = event.payload.get("compiled_constituencies") {
                    if let Ok(v) = serde_json::from_value(c.clone()) {
                        self.compiled_constituencies = v;
                    }
                }
                if let Some(v) = event
                    .payload
                    .get("total_registered")
                    .and_then(|v| v.as_u64())
                {
                    self.total_registered = v;
                }
                if let Some(v) = event.payload.get("total_valid").and_then(|v| v.as_u64()) {
                    self.total_valid = v;
                }
                if let Some(v) = event.payload.get("total_rejected").and_then(|v| v.as_u64()) {
                    self.total_rejected = v;
                }
                if let Some(v) = event.payload.get("total_spoilt").and_then(|v| v.as_u64()) {
                    self.total_spoilt = v;
                }
                if let Some(t) = event.payload.get("candidate_totals") {
                    if let Ok(v) = serde_json::from_value(t.clone()) {
                        self.candidate_totals = v;
                    }
                }
                if let Some(v) = event.payload.get("merkle_root").and_then(|v| v.as_str()) {
                    self.merkle_root = Some(v.to_string());
                }
                self.status = Form34CStatus::Compiled;
            }
            crate::events::EVENT_TALLY_PUBLISHED => {
                self.status = Form34CStatus::Published;
                self.published_at = Some(Utc::now());
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
