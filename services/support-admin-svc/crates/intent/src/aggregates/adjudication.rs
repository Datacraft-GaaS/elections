//! AdjudicationEvent aggregate — M27 ballot adjudication with photo evidence.

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::catalogue::AdjudicationDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjudicationEvent {
    pub stream_id: Uuid,
    pub version: i64,
    pub ballot_id: String,
    pub station_code: String,
    pub contest_id: String,
    pub adjudicator_id: String,
    pub reason_code: String,
    pub decision: Option<AdjudicationDecision>,
    pub photo_evidence_ref: Option<String>,
}

impl AdjudicationEvent {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            ballot_id: String::new(),
            station_code: String::new(),
            contest_id: String::new(),
            adjudicator_id: String::new(),
            reason_code: String::new(),
            decision: None,
            photo_evidence_ref: None,
        }
    }
}

impl Aggregate for AdjudicationEvent {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        if event.event_type == crate::events::EVENT_ADJUDICATION {
            macro_rules! str_field {
                ($key:literal => $dst:expr) => {
                    if let Some(v) = event.payload.get($key).and_then(|v| v.as_str()) {
                        $dst = v.to_string();
                    }
                };
            }
            str_field!("ballot_id" => self.ballot_id);
            str_field!("station_code" => self.station_code);
            str_field!("contest_id" => self.contest_id);
            str_field!("adjudicator_id" => self.adjudicator_id);
            str_field!("reason_code" => self.reason_code);
            if let Some(v) = event.payload.get("decision").and_then(|v| v.as_str()) {
                self.decision = serde_json::from_str(&format!("\"{v}\"")).ok();
            }
            if let Some(v) = event
                .payload
                .get("photo_evidence_ref")
                .and_then(|v| v.as_str())
            {
                self.photo_evidence_ref = Some(v.to_string());
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
