//! Cross-sensor iris template alignment.
//!
//! Normalises iris templates between NIR (700-1000 nm) and visible
//! (400-700 nm) sensor modalities via a learned linear projection,
//! enabling cross-wavelength matching without re-enrolment.

pub mod aligner;
pub mod projection;

pub use aligner::{AlignError, CrossSensorAligner};
