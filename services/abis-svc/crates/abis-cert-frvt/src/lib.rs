//! eel-abis-cert-frvt: NIST FRVT participation infrastructure.
//!
//! Packages template extraction, verification pipeline, and report card
//! generation matching the FRVT 1:1, 1:N, Quality, and PAD tracks.
#![warn(missing_docs)]

pub mod frvt_1v1;
pub mod frvt_1vn;
pub mod frvt_pad;
pub mod frvt_quality;

pub use frvt_1v1::{Frvt1v1Error, Frvt1v1Report, Frvt1v1Submission};
pub use frvt_1vn::{Frvt1vNReport, Frvt1vNSubmission};
pub use frvt_pad::{FrvtPadReport, FrvtPadSubmission};
pub use frvt_quality::{FrvtQualityReport, FrvtQualitySubmission};
