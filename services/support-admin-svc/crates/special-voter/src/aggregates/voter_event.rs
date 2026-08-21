//! SpecialVoterEvent aggregate — M25 domain model, data class A (audit).

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialVoterEvent {
    pub stream_id: Uuid,
    pub version: i64,
    pub facility_id: Uuid,
    pub category: String,
    pub event_type: String,
    pub officer_id: String,
    pub witness_ids: Vec<String>,
    pub reason_code: Option<String>,
    pub merkle_node_id: Option<String>,
    pub event_at: Option<String>,
}

impl SpecialVoterEvent {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            facility_id: Uuid::nil(),
            category: String::new(),
            event_type: String::new(),
            officer_id: String::new(),
            witness_ids: Vec::new(),
            reason_code: None,
            merkle_node_id: None,
            event_at: None,
        }
    }
}

impl Aggregate for SpecialVoterEvent {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        if event.event_type == crate::events::EVENT_SPECIAL_VOTER_EVENT {
            if let Some(v) = event.payload.get("facility_id").and_then(|v| v.as_str()) {
                self.facility_id = v.parse().unwrap_or(Uuid::nil());
            }
            if let Some(v) = event.payload.get("category").and_then(|v| v.as_str()) {
                self.category = v.to_string();
            }
            if let Some(v) = event.payload.get("event_type").and_then(|v| v.as_str()) {
                self.event_type = v.to_string();
            }
            if let Some(v) = event.payload.get("officer_id").and_then(|v| v.as_str()) {
                self.officer_id = v.to_string();
            }
            if let Some(arr) = event.payload.get("witness_ids") {
                if let Ok(ids) = serde_json::from_value::<Vec<String>>(arr.clone()) {
                    self.witness_ids = ids;
                }
            }
            if let Some(v) = event.payload.get("reason_code").and_then(|v| v.as_str()) {
                self.reason_code = Some(v.to_string());
            }
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
