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

//! Error types for bulk operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BulkOpsError {
    #[error("operation not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("operation is not in a resumable state")]
    NotResumable,

    #[error("operation already completed or cancelled")]
    AlreadyTerminal,

    #[error("item processing failed: {0}")]
    ItemFailed(String),

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
