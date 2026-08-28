//! special-voter: domain aggregates, events, workflows, and guardrails for M25.
//!
//! WP-5.5 — M25 special voter workflows: remand prisoners, hospital patients,
//! KDF/GSU/Prison Service, refugees (conditional), and IDPs.
//! Elections Act ss.5, 44, 44A, 54, 72.
#![warn(missing_docs)]

pub mod aggregates;
pub mod http;
/// Typed models for support-special-voter contract.
pub mod spec;
pub mod events;
pub mod guards;
pub mod workflows;
