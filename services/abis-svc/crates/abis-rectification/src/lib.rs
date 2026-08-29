//! `eel-abis-rectification`: subject-driven biometric correction workflow.
//!
//! Requests flow through: submission -> triage -> evidence_pending -> approved/rejected -> completed.
//! SLA is 30 days from submission. SLA sweep runs every 15 minutes.
#![warn(missing_docs)]

pub mod api;
pub mod domain;
pub mod error;
pub mod policy;
pub mod scheduler;
pub mod workflow;

pub use domain::{EvidenceType, RectificationRequest, RectificationRequestType, RectificationState, SlaStatus};
pub use error::RectificationError;
