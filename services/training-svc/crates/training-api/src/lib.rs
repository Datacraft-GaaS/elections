//! Training API HTTP surface for LMS, accreditation, medical, and CMS.
#![warn(missing_docs)]
/// Exam domain.
pub mod exam;
/// HTTP handlers.
pub mod handlers;
/// Domain models.
pub mod models;
/// Router composition.
pub mod router;
/// Typed spec handlers.
pub mod spec;
/// In-memory spec state (swap boundary).
pub mod spec_state;
/// Store trait.
pub mod store;
