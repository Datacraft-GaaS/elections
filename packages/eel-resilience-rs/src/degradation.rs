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

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Operating mode for a service. Every service implements all three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradationMode {
    /// Live queries, real-time updates. Normal operation.
    Normal,
    /// Serve last-known-good signed snapshots; disable expensive queries.
    Degraded,
    /// Reject all writes, serve cached content, log every request, alert IR.
    ReadOnlyLockdown,
}

impl Default for DegradationMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Shared flag that any request path can query.
#[derive(Debug, Clone)]
pub struct DegradationFlag(Arc<RwLock<DegradationMode>>);

impl DegradationFlag {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(DegradationMode::Normal)))
    }

    pub fn with_mode(mode: DegradationMode) -> Self {
        Self(Arc::new(RwLock::new(mode)))
    }

    pub async fn get(&self) -> DegradationMode {
        *self.0.read().await
    }

    pub async fn set(&self, mode: DegradationMode) {
        *self.0.write().await = mode;
    }

    pub async fn is_write_allowed(&self) -> bool {
        !matches!(*self.0.read().await, DegradationMode::ReadOnlyLockdown)
    }
}

impl Default for DegradationFlag {
    fn default() -> Self {
        Self::new()
    }
}
