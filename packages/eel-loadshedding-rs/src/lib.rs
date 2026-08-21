//! CoDel-style priority load shedder for the eel platform (ADR-0009, ADR-0011).
//!
//! Wraps any tower::Service with configurable load shedding:
//! - Priority classes: Admin > AuthenticatedUser > AccreditedMachine > Anonymous
//! - CoDel algorithm: targets latency budget, sheds lowest priority first
//! - Returns 503 with Retry-After and problem+json on shed
//! - Metrics: shed_count per priority, breach count, queue depth

pub mod error;
pub mod priority;
pub mod metrics;
pub mod shedder;
pub mod layer;

pub use error::ShedError;
pub use priority::Priority;
pub use metrics::{ShedMetrics, MetricsSnapshot};
pub use shedder::{CoDelConfig, CoDelShedder};
pub use layer::{LoadSheddingLayer, LoadSheddingService};
