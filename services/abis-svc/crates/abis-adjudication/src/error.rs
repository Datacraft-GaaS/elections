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

//! Error types for the adjudication crate.

use thiserror::Error;

use crate::audit::AuditError;
use crate::fsm::FsmError;

#[derive(Debug, Error)]
pub enum AdjudicationError {
    #[error("FSM: {0}")]
    Fsm(#[from] FsmError),

    #[error("audit chain: {0}")]
    Audit(#[from] AuditError),

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),

    #[error("case not found: {0}")]
    CaseNotFound(uuid::Uuid),

    #[error("no eligible reviewer found for tier {0:?}")]
    NoReviewerAvailable(crate::domain::ReviewerTier),
}
