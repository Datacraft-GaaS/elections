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
