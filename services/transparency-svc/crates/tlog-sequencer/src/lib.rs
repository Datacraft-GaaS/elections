//! tlog-sequencer — domain-tagged Merkle sequencer for the eel transparency log.
//!
//! Targets 50k leaves/s local throughput. Back-pressures producers with 429
//! + Retry-After when the rate limit is exceeded.
//!
//! Three background tasks produce STHs at 1s / 60s / 5min cadences.

pub mod db;
pub mod rate;
pub mod sequencer;
pub mod sth_producer;

pub use sequencer::{AppendResult, Sequencer};
pub use sth_producer::SthProducer;

#[derive(Debug, thiserror::Error)]
pub enum SequencerError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("domain error: {0}")]
    Domain(#[from] tlog_domain::DomainError),
}
