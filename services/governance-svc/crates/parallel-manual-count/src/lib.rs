//! parallel-manual-count crate.
#![warn(missing_docs)]
pub mod api;
pub mod divergence;
pub mod ingest;
pub mod plan;
pub mod rla_feed;
pub mod sample;

pub use divergence::{compute_divergence, DivergenceReport};
pub use plan::SamplePlan;
pub use sample::{sample_stations, SampledStation, Stratum};
