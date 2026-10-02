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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdhocError {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("preview: {0}")]
    Preview(String),
    #[error("publish: {0}")]
    Publish(String),
    #[error("render: {0}")]
    Render(String),
    #[error("query too large: {rows} rows exceeds limit {limit}")]
    TooManyRows { rows: usize, limit: usize },
    #[error("statement timeout")]
    Timeout,
    #[error("memory cap exceeded: {bytes} bytes")]
    MemoryCap { bytes: usize },
}

pub type Result<T, E = AdhocError> = std::result::Result<T, E>;
