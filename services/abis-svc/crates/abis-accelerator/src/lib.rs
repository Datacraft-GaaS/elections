//! `eel-abis-accelerator`: GPU/FPGA offload interfaces and CPU fallback for biometric matching.
//!
//! Trait-based interfaces for hardware accelerators. The dispatcher selects an accelerator
//! based on gallery size, latency budget, and energy budget.
#![warn(missing_docs)]

pub mod cpu_fallback;
pub mod dispatcher;
pub mod domain;
pub mod error;
pub mod fpga;
pub mod gpu;

pub use cpu_fallback::CpuFallbackMatcher;
pub use domain::{AcceleratorCapabilities, AcceleratorKind, MatchQuery, Score};
pub use error::AcceleratorError;
pub use fpga::FpgaMatcher;
pub use gpu::GpuMatcher;
