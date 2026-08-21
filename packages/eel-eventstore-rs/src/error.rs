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
