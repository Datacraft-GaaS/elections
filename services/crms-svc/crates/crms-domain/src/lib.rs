//! crms-domain: core types, quota engine, clearance FSM, and symbol checks.
//!
//! WP-4.11 — M12 (CRMS), M45 (Party Registration), M46 (Primaries Oversight),
//! M47 (Nomination Fee Payment), M67 (Symbol/Colour Registry).
#![warn(missing_docs)]

pub mod candidate;
pub mod nomination;
pub mod clearance;
pub mod quota;
pub mod gazette;
pub mod ballot;
pub mod party;
pub mod coalition;
pub mod primaries;
pub mod payment;
pub mod symbol;
pub mod events;
