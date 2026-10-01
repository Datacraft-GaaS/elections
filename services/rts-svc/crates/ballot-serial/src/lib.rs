//! Ballot-serial chain crate: event-sourced BallotSerial aggregate,
//! reconciliation arithmetic with holds, and the axum API sub-router.
//! Part of rts-svc WP-3.3 (M63).
#![warn(missing_docs)]

pub mod aggregate;
pub mod api;
pub mod events;
pub mod reconciliation;
pub mod store;
