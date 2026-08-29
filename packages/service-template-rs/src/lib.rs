//! Shared conformance test harness for eel Rust services.
//!
//! Generated services call the helpers from `tests/conformance/`
//! via the `conformance-suite` feature.
#![warn(missing_docs)]

#[cfg(feature = "conformance-suite")]
pub mod conformance;
