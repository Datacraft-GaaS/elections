//! Iris image quality assessment per ISO/IEC 29794-6.
//!
//! Reference: ISO/IEC 29794-6:2021. Biometric sample quality.
//! Part 6: Iris image data.

pub mod metrics;
pub mod policy;
pub mod quality;

pub use policy::{QualityPolicy, PolicyError};
pub use quality::{QualityReport, QualityError};
