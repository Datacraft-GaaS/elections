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

//! RejectionRateAlert aggregate — M27, statistical outlier detection for ECC.
//! Z-score threshold: |z| >= 3.0 triggers alert to ECC (configurable per election).

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default Z-score magnitude threshold for triggering an ECC alert.
pub const DEFAULT_Z_THRESHOLD: f64 = 3.0;

/// Target maximum national rejected ballot rate.
pub const TARGET_REJECTION_RATE: f64 = 0.01;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionRateAlert {
    pub stream_id: Uuid,
    pub version: i64,
    pub station_code: String,
    pub contest_id: String,
    pub rejection_rate: f64,
    pub national_mean: f64,
    pub z_score: f64,
    pub alerted_at: String,
    pub acknowledged: bool,
}

impl RejectionRateAlert {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            station_code: String::new(),
            contest_id: String::new(),
            rejection_rate: 0.0,
            national_mean: 0.0,
            z_score: 0.0,
            alerted_at: String::new(),
            acknowledged: false,
        }
    }

    /// Compute z-score from per-station rate, national mean, and stddev.
    pub fn compute_z_score(station_rate: f64, mean: f64, stddev: f64) -> f64 {
        if stddev == 0.0 {
            return 0.0;
        }
        (station_rate - mean) / stddev
    }

    /// Return true if this z-score breaches the alert threshold.
    pub fn is_outlier(z: f64) -> bool {
        z.abs() >= DEFAULT_Z_THRESHOLD
    }
}

impl Aggregate for RejectionRateAlert {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        if event.event_type == crate::events::EVENT_REJECTION_RATE_ALERT {
            macro_rules! str_field {
                ($key:literal => $dst:expr) => {
                    if let Some(v) = event.payload.get($key).and_then(|v| v.as_str()) {
                        $dst = v.to_string();
                    }
                };
            }
            str_field!("station_code" => self.station_code);
            str_field!("contest_id" => self.contest_id);
            str_field!("alerted_at" => self.alerted_at);
            if let Some(v) = event.payload.get("rejection_rate").and_then(|v| v.as_f64()) {
                self.rejection_rate = v;
            }
            if let Some(v) = event.payload.get("national_mean").and_then(|v| v.as_f64()) {
                self.national_mean = v;
            }
            if let Some(v) = event.payload.get("z_score").and_then(|v| v.as_f64()) {
                self.z_score = v;
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
