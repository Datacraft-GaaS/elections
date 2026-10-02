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

pub const CONTAINER_DISPATCHED: &str = "eel.diaspora.container_dispatched";
pub const CONTAINER_RECEIVED: &str = "eel.diaspora.container_received";
pub const SEAL_BROKEN: &str = "eel.diaspora.seal_broken";
pub const RECONCILIATION_COMPLETE: &str = "eel.diaspora.reconciliation_complete";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDispatchedPayload {
    pub container_id: String,
    pub actor_id: String,
    pub location: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealBrokenPayload {
    pub container_id: String,
    pub seal_id: String,
    pub broken_by: String,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}
