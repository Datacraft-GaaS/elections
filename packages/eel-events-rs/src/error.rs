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
pub enum EventError {
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    #[error("signing error: {0}")]
    Signing(String),

    #[error("canonical JSON error: {0}")]
    Canon(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("schema error: {0}")]
    Schema(String),

    #[error("signature verification failed — event may be tampered")]
    TamperDetected,

    #[error("duplicate event id: {0}")]
    Duplicate(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("concurrency error: {0}")]
    Concurrency(String),

    #[error("cryptographic error: {0}")]
    Crypto(String),
}
