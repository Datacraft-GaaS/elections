//! `budget` HTTP service for M54 budget-expenditure (WP-5.6).
//!
//! Manages budgets, budget lines, procurement records, contracts,
//! payment vouchers, CoB reports, and OAG audit packages for support-admin-svc.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod handlers;
pub mod routes;
