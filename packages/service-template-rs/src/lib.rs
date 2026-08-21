//! Shared conformance test harness for eel Rust services.
//!
//! Generated services call the helpers from `tests/conformance/`
//! via the `conformance-suite` feature.

#[cfg(feature = "conformance-suite")]
pub mod conformance;
