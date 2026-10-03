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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchlistClassification {
    Exclusion,
    DuplicateAlert,
    VipFlag,
    FraudHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    pub watchlist_id: String,
    pub name: String,
    pub description: Option<String>,
    pub modalities: Vec<String>,
    pub max_size: u32,
    pub current_size: u32,
    pub classification: WatchlistClassification,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub suspended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistEntry {
    pub entry_id: String,
    pub watchlist_id: String,
    pub subject_ref: String,
    pub modality: String,
    pub template_hash: String,
    pub hit_count: u32,
    pub last_hit_at: Option<DateTime<Utc>>,
    pub added_at: DateTime<Utc>,
    pub sunset_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl WatchlistEntry {
    pub fn is_sunsetted(&self) -> bool {
        self.sunset_at.is_some_and(|t| t <= Utc::now())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit {
    pub entry_id: String,
    pub watchlist_id: String,
    pub watchlist_name: String,
    pub subject_ref: String,
    pub modality: String,
    pub similarity: f32,
    pub hit_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalProbe {
    pub probe_id: String,
    pub template_hashes: Vec<(String, String)>,
}
