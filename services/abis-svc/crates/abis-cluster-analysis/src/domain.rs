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

//! Domain types for cluster analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A confirmed match edge between two subjects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchEdge {
    pub subject_a: Uuid,
    pub subject_b: Uuid,
    pub score: f64,
    pub confirmed_at: DateTime<Utc>,
}

/// Resolution action for a cluster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterResolution {
    Pending,
    MergedIntoOne,
    SplitIntoGroups,
    FalseAlarm,
}

/// A duplicate cluster record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRecord {
    pub id: Uuid,
    pub member_ids: Vec<Uuid>,
    pub resolution: ClusterResolution,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub edge_count: usize,
    pub notes: Option<String>,
}

impl ClusterRecord {
    pub fn new(member_ids: Vec<Uuid>, edge_count: usize) -> Self {
        Self {
            id: Uuid::now_v7(),
            member_ids,
            resolution: ClusterResolution::Pending,
            detected_at: Utc::now(),
            resolved_at: None,
            edge_count,
            notes: None,
        }
    }
}
