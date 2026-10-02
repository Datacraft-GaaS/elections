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

//! Public contract models (queue/transparency.yaml).

use serde::{Deserialize, Serialize};

/// Public per-station queue status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub station_code: String,
    pub estimated_wait_minutes: u32,
    pub queue_length: u32,
    pub last_updated_at: String,
    pub is_open: bool,
    pub observation_count: u32,
}

/// Request body for submitting a queue observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueObservationSubmission {
    pub station_code: String,
    pub observed_queue_length: u32,
    pub observed_at: String,
    pub observer_id_hash: String,
}

/// Public queue observation view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueObservation {
    pub observation_id: String,
    pub station_code: String,
    pub observed_queue_length: u32,
    pub observed_at: String,
    pub submitted_at: String,
}

/// Public county-level queue analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyQueueAnalytics {
    pub county_code: String,
    pub generated_at: String,
    pub station_count: u32,
    pub stations_open: u32,
    pub avg_wait_minutes: f64,
    pub max_wait_minutes: u32,
    pub total_observations: u32,
    pub peak_hour: Option<String>,
}
