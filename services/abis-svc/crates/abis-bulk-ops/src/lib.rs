//! `eel-abis-bulk-ops`: batch biometric operations with idempotent resumption.
//!
//! Supports 7 operation types: enrol, dedup, delete, export, rewrap, import, retire.
//! Each operation is resumable from a progress checkpoint. Failure manifests are
//! written as CSV.
#![warn(missing_docs)]

pub mod api;
pub mod dedup;
pub mod delete;
pub mod dispatcher;
pub mod domain;
pub mod enrol;
pub mod error;
pub mod export;
pub mod import;
pub mod progress;
pub mod retire;
pub mod rewrap;

pub use domain::{BulkOperation, BulkOpState, BulkOpType, FailureRecord, ProgressSnapshot};
pub use error::BulkOpsError;
