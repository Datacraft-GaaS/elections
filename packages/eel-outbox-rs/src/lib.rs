//! Transactional outbox for the eel platform (WP-1.2).
//!
//! The `Outbox` type is the **sole** mechanism for publishing events to
//! Kafka from any eel service. It inserts a row into the Postgres `outbox`
//! table inside the caller's transaction; Debezium captures the row via
//! CDC and publishes it atomically.
//!
//! # Why an outbox?
//!
//! Publishing directly to Kafka from application code creates a dual-write
//! race: the business transaction may commit while the Kafka produce fails
//! (or vice versa). The outbox pattern closes this gap by making the
//! database the source of truth; Debezium delivers exactly-once semantics
//! to Kafka via idempotent produce + transactional offsets.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use eel_outbox_rs::{Outbox, OutboxEnvelope};
//! use serde_json::json;
//!
//! async fn example(pool: sqlx::PgPool) -> Result<(), eel_outbox_rs::OutboxError> {
//!     let outbox = Outbox::new(pool.clone());
//!     outbox.migrate().await?;
//!
//!     let envelope = OutboxEnvelope {
//!         id: "01950f8c-0000-7000-8000-000000000001".into(),
//!         event_type: "eel.rts.form34a_captured.v1".into(),
//!         source: "spiffe://eel.local/service/rts-backend".into(),
//!         subject: Some("form-34a/47-0001-001".into()),
//!         time: "2027-08-15T15:30:00.123Z".into(),
//!         eel_stream_id: "01950f8c-0000-7000-8000-000000000001".into(),
//!         eel_seq: 1,
//!         eel_hlc: 1723731000123,
//!         eel_prev_hash: None,
//!         eel_correlation_id: None,
//!         eel_causation_id: None,
//!         eel_producer_key_id: None,
//!         eel_signature: None,
//!         eel_election_id: Some("2027-general".into()),
//!         eel_data_class: Some("B".into()),
//!         data: json!({ "form_id": "47-0001-001" }),
//!         topic: "eel.rts.form34a.v1".into(),
//!         kafka_key: "01950f8c-0000-7000-8000-000000000001".into(),
//!         aggregate_type: "form34a".into(),
//!         aggregate_id: "01950f8c-0000-7000-8000-000000000001".into(),
//!     };
//!
//!     let mut tx = pool.begin().await.unwrap();
//!     outbox.emit(&mut tx, &envelope).await?;
//!     tx.commit().await.unwrap();
//!     Ok(())
//! }
//! ```
#![warn(missing_docs)]

pub mod envelope;
pub mod error;
pub mod outbox;
pub mod schema;

#[cfg(feature = "minio")]
pub mod minio;

pub use envelope::OutboxEnvelope;
pub use error::OutboxError;
pub use outbox::{Outbox, MAX_PAYLOAD_BYTES};
