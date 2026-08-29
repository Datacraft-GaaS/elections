//! `eel-abis-enrolment`: enrolment lifecycle and 1:N deduplication pipeline.
//!
//! Drives biometric enrolment jobs from Received through quality check,
//! 1:N deduplication, adjudication routing, and final gallery placement.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod events;
pub mod orchestrator;
pub mod pipeline;
pub mod repository;

pub use domain::{
    CapturedTemplate, DedupDecision, DedupResult, EnrolmentDecision, EnrolmentJob,
    EnrolmentSource, EnrolmentStatus, FingerPosition, Modality, PadResult, ProbableMatch,
    QualityReport,
};
pub use error::EnrolmentError;
pub use pipeline::{apply_dedup_result, apply_quality_report, finalise, ingest, route_decision};
