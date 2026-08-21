//! SpoiltBallotEvent aggregate — M27, Elections Act s.54 spoilt ballot replacement.

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoiltBallotEvent {
    pub stream_id: Uuid,
    pub version: i64,
    pub station_code: String,
    pub contest_id: String,
    pub reason_code: String,
    pub officer_id: String,
    pub witness_ids: Vec<String>,
    pub replacement_issued: bool,
}

impl SpoiltBallotEvent {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            station_code: String::new(),
            contest_id: String::new(),
            reason_code: String::new(),
            officer_id: String::new(),
            witness_ids: Vec::new(),
            replacement_issued: false,
        }
    }
}

impl Aggregate for SpoiltBallotEvent {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        if event.event_type == crate::events::EVENT_SPOILT_BALLOT {
            macro_rules! str_field {
                ($key:literal => $dst:expr) => {
                    if let Some(v) = event.payload.get($key).and_then(|v| v.as_str()) {
                        $dst = v.to_string();
                    }
                };
            }
            str_field!("station_code" => self.station_code);
            str_field!("contest_id" => self.contest_id);
            str_field!("reason_code" => self.reason_code);
            str_field!("officer_id" => self.officer_id);
            if let Some(arr) = event.payload.get("witness_ids") {
                if let Ok(ids) = serde_json::from_value::<Vec<String>>(arr.clone()) {
                    self.witness_ids = ids;
                }
            }
            if let Some(v) = event
                .payload
                .get("replacement_issued")
                .and_then(|v| v.as_bool())
            {
                self.replacement_issued = v;
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
