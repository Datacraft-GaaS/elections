//! Event store with optimistic concurrency, snapshot/replay, and exactly-once
//! Kafka consumer helper for the eel platform (WP-1.2).
//!
//! # Quick start
//!
//! ```no_run
//! use eel_eventstore_rs::{EventStore, EventStoreError};
//!
//! # async fn _doctest() -> Result<(), EventStoreError> {
//! let pool = sqlx::PgPool::connect("postgres://localhost/eel").await?;
//! let store = EventStore::new(pool);
//! store.migrate().await?;
//! # Ok(())
//! # }
//! ```
#![warn(missing_docs)]

pub mod aggregate;
pub mod error;
pub mod replay;
pub mod schema;
pub mod store;

#[cfg(feature = "kafka")]
pub mod exactly_once;

pub use aggregate::{Aggregate, AggregateSnapshot, StoredEvent};
pub use error::EventStoreError;
pub use replay::{EventReplay, SnapshotTailReplay};
pub use schema::CREATE_SCHEMA_SQL;
pub use store::EventStore;
