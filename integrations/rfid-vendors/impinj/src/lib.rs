//! eel-rfid-impinj: Impinj Speedway R700 vendor adapter.
//!
//! This crate provides:
//! - `adapter::ImpinjAdapter` — HTTP client wrapping the py/rfid-reader-driver endpoint
//!   with circuit-breaker resilience via eel-resilience-rs.
//! - `double::MockImpinj` — in-memory double for dev/chaos testing with tamper injection.
#![warn(missing_docs)]

pub mod adapter;
pub mod double;

pub use adapter::ImpinjHandler;
pub use double::MockImpinj;
