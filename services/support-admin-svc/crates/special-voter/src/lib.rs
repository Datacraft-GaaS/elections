//! special-voter: domain aggregates, events, workflows, and guardrails for M25.
//!
//! WP-5.5 — M25 special voter workflows: remand prisoners, hospital patients,
//! KDF/GSU/Prison Service, refugees (conditional), and IDPs.
//! Elections Act ss.5, 44, 44A, 54, 72.

pub mod aggregates;
pub mod http;
pub mod events;
pub mod guards;
pub mod workflows;
