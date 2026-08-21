//! IDPReassignment aggregate — M25 domain model, data class B.

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDPReassignment {
    pub stream_id: Uuid,
    pub version: i64,
    /// Encrypted voter ID (PII; never logged unencrypted).
    pub voter_id_enc: String,
    pub original_station_code: String,
    pub idp_facility_id: Uuid,
    pub displacement_source: String,
    pub effective_date: String,
    pub reassigned_at: Option<String>,
}

impl IDPReassignment {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            voter_id_enc: String::new(),
            original_station_code: String::new(),
            idp_facility_id: Uuid::nil(),
            displacement_source: String::new(),
            effective_date: String::new(),
            reassigned_at: None,
        }
    }
}

impl Aggregate for IDPReassignment {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        if event.event_type == crate::events::EVENT_IDP_REASSIGNMENT_RECORDED {
            if let Some(v) = event.payload.get("voter_id_enc").and_then(|v| v.as_str()) {
                self.voter_id_enc = v.to_string();
            }
            if let Some(v) = event
                .payload
                .get("original_station_code")
                .and_then(|v| v.as_str())
            {
                self.original_station_code = v.to_string();
            }
            if let Some(v) = event
                .payload
                .get("idp_facility_id")
                .and_then(|v| v.as_str())
            {
                self.idp_facility_id = v.parse().unwrap_or(Uuid::nil());
            }
            if let Some(v) = event
                .payload
                .get("displacement_source")
                .and_then(|v| v.as_str())
            {
                self.displacement_source = v.to_string();
            }
            if let Some(v) = event.payload.get("effective_date").and_then(|v| v.as_str()) {
                self.effective_date = v.to_string();
            }
            if let Some(v) = event.payload.get("reassigned_at").and_then(|v| v.as_str()) {
                self.reassigned_at = Some(v.to_string());
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
