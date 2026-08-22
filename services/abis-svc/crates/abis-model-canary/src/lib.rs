//! `eel-abis-model-canary`: blue-green model canary rollout with auto-rollback.
//!
//! Supports progressive traffic shifting 0->10->25->50->100% with 15-min hold
//! periods between steps. Deterministic per-subject assignment via FNV-1a hash.

pub mod canary;
pub mod dispatcher;
pub mod domain;
pub mod error;
pub mod fairness_rollback;
pub mod monitor;
pub mod promote;
pub mod rollback;

pub use domain::{CanaryMetrics, CanaryRecord, CanaryState, ModelCard, RollbackReason};
pub use error::CanaryError;
