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
pub enum EventStoreError {
    #[error("optimistic concurrency conflict: expected version {expected}, actual {actual}")]
    ConcurrencyConflict { expected: i64, actual: i64 },

    #[error("stream not found: {stream_id}")]
    StreamNotFound { stream_id: uuid::Uuid },

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HLC parse error: {0}")]
    HlcParse(#[from] eel_hlc_rs::HlcError),

    #[cfg(feature = "kafka")]
    #[error("kafka error: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),
}
