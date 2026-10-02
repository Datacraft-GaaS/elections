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

//! Error types for model canary.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanaryError {
    #[error("canary not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("canary already active for model {0}")]
    AlreadyActive(uuid::Uuid),

    #[error("SLA breach: {0}")]
    SlaBreach(String),

    #[error("invalid traffic pct {0}: must be 0-100")]
    InvalidTrafficPct(u8),

    #[error("canary is not in InProgress state")]
    NotInProgress,

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
