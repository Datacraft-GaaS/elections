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

//! PreMarkValidationEvent aggregate — M27, on-device pre-mark validation.
//! Image must be deleted <= 1 s post-inference (model.IMAGE_DELETE_DEADLINE_MS).

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreMarkValidationEvent {
    pub stream_id: Uuid,
    pub version: i64,
    pub station_code: String,
    pub contest_id: String,
    pub inference_outcome: String,
    pub model_version: String,
    pub device_id: String,
    pub image_delete_elapsed_ms: u32,
}

impl PreMarkValidationEvent {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            station_code: String::new(),
            contest_id: String::new(),
            inference_outcome: String::new(),
            model_version: String::new(),
            device_id: String::new(),
            image_delete_elapsed_ms: 0,
        }
    }
}

impl Aggregate for PreMarkValidationEvent {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        if event.event_type == crate::events::EVENT_PREMARK_VALIDATION {
            macro_rules! field_str {
                ($key:literal => $dst:expr) => {
                    if let Some(v) = event.payload.get($key).and_then(|v| v.as_str()) {
                        $dst = v.to_string();
                    }
                };
            }
            field_str!("station_code" => self.station_code);
            field_str!("contest_id" => self.contest_id);
            field_str!("inference_outcome" => self.inference_outcome);
            field_str!("model_version" => self.model_version);
            field_str!("device_id" => self.device_id);
            if let Some(v) = event
                .payload
                .get("image_delete_elapsed_ms")
                .and_then(|v| v.as_u64())
            {
                self.image_delete_elapsed_ms = v as u32;
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
