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

use crate::error::BatchError;
use bvr_domain::ids::uuid7str;
use bvr_domain::snapshot::RegisterSnapshot;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Export configuration.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub election_scope: String,
    /// Resume from this checkpoint token if set.
    pub resume_from: Option<String>,
    /// Max rows per station extract file.
    pub batch_size: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            election_scope: "2027-general".into(),
            resume_from: None,
            batch_size: 10_000,
        }
    }
}

/// Checkpoint state for resumable export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCheckpoint {
    pub export_id: String,
    pub election_scope: String,
    pub county_code: String,
    pub station_id: String,
    pub rows_written: u64,
    pub created_at: chrono::DateTime<Utc>,
}

impl ExportCheckpoint {
    pub fn token(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.export_id, self.county_code, self.station_id, self.rows_written
        )
    }
}

/// Drive a register export to completion. Returns the final snapshot.
/// This is a stub. The real implementation streams rows from the DB
/// per county/station partition, writing signed per-station extract files
/// to MinIO, with checkpoint rows in the database for resumability.
pub async fn run_export(config: ExportConfig) -> Result<RegisterSnapshot, BatchError> {
    let snapshot_id = uuid7str();
    // Stub: in production this streams from sqlx via cursor
    Ok(RegisterSnapshot {
        snapshot_id,
        election_scope: config.election_scope,
        station_count: 0,
        voter_count: 0,
        merkle_root: "0".repeat(64),
        signed_by: "hsm-key-1".into(),
        created_at: Utc::now(),
        checkpoint: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn export_returns_snapshot() {
        let snap = run_export(ExportConfig::default()).await.unwrap();
        assert_eq!(snap.election_scope, "2027-general");
        assert_eq!(snap.merkle_root.len(), 64);
    }
}
