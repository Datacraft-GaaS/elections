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
pub enum OutboxError {
    #[error("payload exceeds 1 MiB limit ({size} bytes); upload blobs to MinIO first")]
    PayloadTooLarge { size: usize },

    #[error("MinIO blob upload failed: {0}")]
    BlobUpload(String),

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("invalid envelope: {0}")]
    InvalidEnvelope(String),
}
