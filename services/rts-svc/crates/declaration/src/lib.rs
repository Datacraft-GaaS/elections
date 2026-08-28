//! Declaration crate: event-sourced Declaration, CertificateOfElection,
//! PetitionWindowTracker, and the presidential run-off/tie rules engine.
//! Part of rts-svc WP-3.3 (M51).

pub mod aggregate;
pub mod api;
pub mod events;
pub mod gazette;
pub mod quorum;
pub mod rules;

#[cfg(test)]
mod proptest_rules;
