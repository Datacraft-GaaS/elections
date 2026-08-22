//! `eel-field-mobile-telemetry-ingest`
//!
//! Server-side telemetry ingestion pipeline for field mobile devices.
//! Implements: signal ingest, HMAC verification, hash chain, replay defence,
//! batch ingest, activity frame downsampling, geo-dwell computation,
//! alert engine, daily Merkle snapshots, and Prometheus metrics.
//!
//! Ref: docs/features/mobile-telemetry-and-observability.md

pub mod alert_engine;
pub mod api;
pub mod batch;
pub mod domain;
pub mod dwell_computer;
pub mod error;
pub mod events;
pub mod frames;
pub mod hash_chain;
pub mod hmac_verify;
pub mod ingest;
pub mod merkle_snapshot;
pub mod metrics;
pub mod replay_defence;
pub mod repository;

pub use domain::{
    AlertClass, ExitReason, MotionClass, Severity, SignalClass, TelemetryActivityFrame,
    TelemetryAlert, TelemetryDevice, TelemetryGeoDwell, TelemetrySession, TelemetrySignal,
    TelemetrySnapshot,
};
pub use error::TelemetryError;
pub use hash_chain::{compute_this_hash, verify_chain};
