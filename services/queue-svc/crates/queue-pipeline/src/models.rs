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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSnapshot {
    pub station_code: String,
    pub kit_throughput_per_hour: u32,
    pub clerk_report_count: u32,
    pub cv_headcount: u32,
    pub qr_scan_count: u32,
    pub estimated_wait_minutes: u32,
    pub disenfranchisement_flag: bool,
    pub s14a_recommended: bool,
    pub fused_at: DateTime<Utc>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSnapshotEvent {
    pub event_type: String,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub snapshot: QueueSnapshot,
}

impl QueueSnapshotEvent {
    pub fn new(snapshot: QueueSnapshot) -> Self {
        Self {
            event_type: "eel.queue.snapshot.v1".to_string(),
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            snapshot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitTimeBreachedEvent {
    pub event_type: String,
    pub station_code: String,
    pub wait_minutes: u32,
    pub threshold_minutes: u32,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisenfranchisementFlagEvent {
    pub event_type: String,
    pub station_code: String,
    pub reason: String,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S14aRecommendedEvent {
    pub event_type: String,
    pub station_code: String,
    pub recommendation_id: Uuid,
    pub hsm_pending: bool,
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureFusionInput {
    pub kit_throughput_per_hour: u32,
    pub clerk_report_count: u32,
    pub cv_headcount: u32,
    pub qr_scan_count: u32,
}
