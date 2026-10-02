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
