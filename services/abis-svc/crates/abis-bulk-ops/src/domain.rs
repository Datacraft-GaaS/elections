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

//! Domain types for bulk operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of bulk operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkOpType {
    Enrol,
    Dedup,
    Delete,
    Export,
    Rewrap,
    Import,
    Retire,
}

impl std::fmt::Display for BulkOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BulkOpType::Enrol => "enrol",
            BulkOpType::Dedup => "dedup",
            BulkOpType::Delete => "delete",
            BulkOpType::Export => "export",
            BulkOpType::Rewrap => "rewrap",
            BulkOpType::Import => "import",
            BulkOpType::Retire => "retire",
        };
        write!(f, "{}", s)
    }
}

/// Lifecycle state of a bulk operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkOpState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// A bulk operation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperation {
    pub id: Uuid,
    pub op_type: BulkOpType,
    pub state: BulkOpState,
    pub total_items: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub skipped: u64,
    pub checkpoint_offset: u64,
    pub manifest_uri: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub operator_id: Uuid,
    pub params: serde_json::Value,
}

impl BulkOperation {
    pub fn new(
        op_type: BulkOpType,
        total_items: u64,
        operator_id: Uuid,
        params: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            op_type,
            state: BulkOpState::Pending,
            total_items,
            succeeded: 0,
            failed: 0,
            skipped: 0,
            checkpoint_offset: 0,
            manifest_uri: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            operator_id,
            params,
        }
    }
}

/// Progress snapshot for resumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSnapshot {
    pub op_id: Uuid,
    pub offset: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub skipped: u64,
}

/// A single failure record for the manifest CSV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub item_id: String,
    pub error: String,
    pub offset: u64,
}
