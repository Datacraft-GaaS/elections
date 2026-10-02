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

//! Error types for physical-seal-binding.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SealBindError {
    #[error("seal_id mismatch: event has '{event}', photo signature has '{photo}'")]
    SealIdMismatch { event: String, photo: String },

    #[error("kit_id mismatch: event has '{event}', photo signature has '{photo}'")]
    KitIdMismatch { event: String, photo: String },

    #[error("kit HSM signature verification failed")]
    SignatureInvalid,

    #[error("photo hash mismatch: expected {expected}, computed {computed}")]
    PhotoHashMismatch { expected: String, computed: String },

    #[error(
        "timestamp out of order: event_ts={event_ts} must be >= photo_captured_at={captured_at}"
    )]
    TimestampOutOfOrder { event_ts: i64, captured_at: i64 },

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
