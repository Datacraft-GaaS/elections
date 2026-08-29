//! bvr-domain crate.
#![warn(missing_docs)]
pub mod canary;
pub mod events;
pub mod ids;
pub mod snapshot;
pub mod voter;

pub use ids::uuid7str;
