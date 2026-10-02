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

//! Nightly job: scan recent adjudication decisions for multi-way collisions.

use chrono::{DateTime, Utc};

use crate::cluster::connected_components;
use crate::domain::{ClusterRecord, MatchEdge};
use crate::error::ClusterError;

/// Scan confirmed match edges since `since` and return duplicate clusters.
pub fn detect_clusters(
    edges: &[MatchEdge],
    since: DateTime<Utc>,
) -> Result<Vec<ClusterRecord>, ClusterError> {
    let recent: Vec<MatchEdge> = edges
        .iter()
        .filter(|e| e.confirmed_at >= since)
        .cloned()
        .collect();
    connected_components(&recent)
}
