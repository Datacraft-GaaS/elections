//! M39 ballot printing chain domain and integration stubs.
#![warn(missing_docs)]
/// domain.
pub mod domain;
/// HTTP handlers for printing contract.
pub mod http;
/// Typed models for printing contract.
pub mod spec;
/// Domain events.
pub mod events;
/// Printer integrations.
pub mod integrations;
