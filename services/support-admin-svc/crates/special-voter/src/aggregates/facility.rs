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

//! SpecialVoterFacility aggregate — M25 domain model, data class B.

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::workflows::{FacilitySessionState, SpecialVoterCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialVoterFacility {
    pub stream_id: Uuid,
    pub version: i64,
    pub category: Option<SpecialVoterCategory>,
    pub county_code: String,
    pub kit_ids: Vec<String>,
    pub presiding_officer_id: String,
    pub voting_date: String,
    pub published_voter_count: u32,
    pub session_state: FacilitySessionState,
    pub violation_pending_ack: bool,
}

impl SpecialVoterFacility {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            category: None,
            county_code: String::new(),
            kit_ids: Vec::new(),
            presiding_officer_id: String::new(),
            voting_date: String::new(),
            published_voter_count: 0,
            session_state: FacilitySessionState::Sealed,
            violation_pending_ack: false,
        }
    }
}

impl Aggregate for SpecialVoterFacility {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        match event.event_type.as_str() {
            crate::events::EVENT_SPECIAL_FACILITY_OPENED => {
                if let Some(v) = event.payload.get("category").and_then(|v| v.as_str()) {
                    self.category = serde_json::from_str(&format!("\"{v}\"")).ok();
                }
                if let Some(v) = event.payload.get("county_code").and_then(|v| v.as_str()) {
                    self.county_code = v.to_string();
                }
                if let Some(v) = event
                    .payload
                    .get("presiding_officer_id")
                    .and_then(|v| v.as_str())
                {
                    self.presiding_officer_id = v.to_string();
                }
                if let Some(v) = event.payload.get("voting_date").and_then(|v| v.as_str()) {
                    self.voting_date = v.to_string();
                }
                if let Some(arr) = event.payload.get("kit_ids") {
                    if let Ok(ids) = serde_json::from_value::<Vec<String>>(arr.clone()) {
                        self.kit_ids = ids;
                    }
                }
                self.session_state = FacilitySessionState::Open;
            }
            crate::events::EVENT_SECRECY_VIOLATION_DETECTED => {
                self.violation_pending_ack = true;
            }
            crate::events::EVENT_SPECIAL_FORM34_TRANSMITTED => {
                self.session_state = FacilitySessionState::Closed;
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
