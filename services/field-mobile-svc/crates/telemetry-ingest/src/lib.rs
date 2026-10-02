// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `eel-field-mobile-telemetry-ingest`
//!
//! Server-side telemetry ingestion pipeline for field mobile devices.
//! Implements: signal ingest, HMAC verification, hash chain, replay defence,
//! batch ingest, activity frame downsampling, geo-dwell computation,
//! alert engine, daily Merkle snapshots, and Prometheus metrics.
//!
//! Ref: docs/features/mobile-telemetry-and-observability.md
#![warn(missing_docs)]

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
