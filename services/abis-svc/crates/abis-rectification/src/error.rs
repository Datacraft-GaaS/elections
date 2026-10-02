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

//! Error types for rectification.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RectificationError {
    #[error("request not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: crate::domain::RectificationState,
        to: crate::domain::RectificationState,
    },

    #[error("evidence required: {0:?}")]
    EvidenceRequired(crate::domain::EvidenceType),

    #[error("ODPC reference required for template_removal requests")]
    OdpcRefRequired,

    #[error("officer assignment required for this request type")]
    OfficerRequired,

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
