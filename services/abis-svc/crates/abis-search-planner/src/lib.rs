//! `eel-abis-search-planner`: cost-based 1:N query planner across shards and accelerators.
//!
//! Plans the cheapest sequence of: demographic_prune -> face_ANN -> fingerprint_exact ->
//! iris_exact, and executes concurrently across shards with early-termination when the
//! confidence threshold is reached.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod estimator;
pub mod executor;
pub mod plan;

pub use domain::{ExecutionPlan, PlanStep, ShardSpec, StepKind};
pub use error::PlannerError;
pub use plan::QueryPlanner;
