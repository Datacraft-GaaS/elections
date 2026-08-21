//! `eel-abis-cluster-analysis`: multi-way duplicate detection via connected-components.
//!
//! Builds a match graph from adjudication-confirmed pairs, then finds connected
//! components to identify duplicate clusters. Runs nightly over the last 30 days.

pub mod api;
pub mod cluster;
pub mod detector;
pub mod domain;
pub mod error;
pub mod resolver;

pub use domain::{ClusterRecord, ClusterResolution, MatchEdge};
pub use error::ClusterError;
pub use cluster::connected_components;
