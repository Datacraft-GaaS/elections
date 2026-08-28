//! intent: domain aggregates, adjudication catalogue, model registry hook,
//! and rejection-rate alerting for M27 intent preservation (WP-5.5).
//!
//! On-device pre-mark visual validation model is registered via M28 registry.
//! Image auto-delete <= 1 second post-inference (FR-M27 privacy constraint).
#![warn(missing_docs)]

pub mod aggregates;
pub mod http;
/// Typed models for intent contract.
pub mod spec;
pub mod catalogue;
pub mod events;
pub mod model;
