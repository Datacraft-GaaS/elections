//! tlog-ingest — Kafka consumer that feeds every platform event into the log.
//!
//! Verifies eel-events-rs signatures; routes tampered events to `<topic>.tamper`.
//! Deduplicates by (source_topic, source_event_id) via PostgreSQL.
//! Back-pressures producers via `TokioSemaphore` bounded by sequencer rate.
#![warn(missing_docs)]

pub mod consumer;
pub mod failures;
pub mod offset_tracker;
pub mod topic_router;

pub use consumer::IngestConsumer;
