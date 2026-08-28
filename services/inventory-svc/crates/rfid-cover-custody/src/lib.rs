//! `eel-inventory-rfid-cover-custody`
//!
//! Tamper-evident RFID cover custody for ballot-box covers.
//! Implements: tag issuance, pairing, read ingest, FSM transitions,
//! BLAKE3 hash chain, tamper alerts, replay defence, and Prometheus metrics.
//!
//! Ref: docs/features/rfid-ballot-box-covers.md

pub mod api;
pub mod domain;
pub mod error;
pub mod events;
pub mod hash_chain;
pub mod metrics;
pub mod officer_sig;
pub mod read_ingest;
pub mod reader;
pub mod replay_defence;
pub mod repository;
pub mod state_machine;
pub mod tag;
pub mod tamper_alert;

pub use domain::{
    CoverRead, CoverState, CoverStateTransition, CoverTag, CoverTamperAlert, ReaderType, Severity,
    TamperState,
};
pub use error::RfidError;
pub use hash_chain::{compute_this_hash, verify_chain};
pub use state_machine::CoverStateMachine;
