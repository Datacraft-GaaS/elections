//! CloudEvents envelope, signing, schema lookup, and consumer for the eel platform.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use eel_events_rs::{EventBuilder, envelope::DataClass};
//!
//! # #[cfg(feature = "test-utils")]
//! # {
//! use eel_events_rs::test_utils::EphemeralKeyProvider;
//! let provider = EphemeralKeyProvider::new();
//! let envelope = EventBuilder::new()
//!     .event_type("eel.rts.form34a_captured.v1")
//!     .source("spiffe://eel.local/service/rts-backend")
//!     .stream_id("01950f8c-0000-7000-0000-000000000001")
//!     .seq(0)
//!     .hlc("1723731000123.0.1")
//!     .data(serde_json::json!({"station_id": "47-001"}))
//!     .data_class(DataClass::B)
//!     .build(&provider)
//!     .unwrap();
//! assert_eq!(envelope.specversion, "1.0");
//! # }
//! ```

pub mod audit_chain;
pub mod builder;
pub mod consumer;
pub mod envelope;
pub mod error;
pub mod karapace;
pub mod pq_leaf;
pub mod spiffe;

#[cfg(feature = "test-utils")]
pub mod test_utils;

pub use builder::EventBuilder;
pub use consumer::{dlq_topic, tamper_topic, EventConsumer, GapBuffer};
pub use envelope::{DataClass, Envelope};
pub use error::EventError;
pub use karapace::KarapaceClient;
pub use spiffe::SpiffeKeyProvider;
