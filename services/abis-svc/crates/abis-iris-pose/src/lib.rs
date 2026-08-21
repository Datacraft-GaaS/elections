//! Off-axis gaze correction for iris images.
//!
//! Estimates gaze angle from pupil/limbus displacement and warps the iris
//! image to compensate for off-axis capture up to approximately +/-20 degrees.

pub mod correction;
pub mod pose;

pub use correction::OffAxisCorrector;
pub use pose::PoseCorrector;
