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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateCount {
    pub candidate_id: String,
    pub manual_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualCountSubmission {
    pub station_id: String,
    pub stratum: String,
    pub submitted_by: String,
    pub submitted_at: String,
    pub candidates: Vec<CandidateCount>,
}

#[derive(Debug, Clone)]
pub struct ManualCountStore {
    pub submissions: Arc<RwLock<HashMap<String, ManualCountSubmission>>>,
}

impl ManualCountStore {
    pub fn new() -> Self {
        ManualCountStore {
            submissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn submit(&self, submission: ManualCountSubmission) -> Result<(), String> {
        let mut guard = self.submissions.write().await;
        if guard.contains_key(&submission.station_id) {
            return Err(format!(
                "duplicate submission for station {}",
                submission.station_id
            ));
        }
        guard.insert(submission.station_id.clone(), submission);
        Ok(())
    }

    pub async fn get(&self, station_id: &str) -> Option<ManualCountSubmission> {
        let guard = self.submissions.read().await;
        guard.get(station_id).cloned()
    }

    pub async fn all(&self) -> Vec<ManualCountSubmission> {
        let guard = self.submissions.read().await;
        guard.values().cloned().collect()
    }

    pub async fn count(&self) -> usize {
        let guard = self.submissions.read().await;
        guard.len()
    }
}

impl Default for ManualCountStore {
    fn default() -> Self {
        Self::new()
    }
}
