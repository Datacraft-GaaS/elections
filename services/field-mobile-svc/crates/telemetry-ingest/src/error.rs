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

//! Error types for the telemetry-ingest crate.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelemetryError {
    // ---- Auth / identity ----
    #[error("SVID rejected: {0}")]
    SvidRejected(String),

    #[error("kit certificate verification failed for device {device_id}")]
    KitCertInvalid { device_id: String },

    // ---- HMAC ----
    #[error("HMAC verification failed for device {device_id}")]
    HmacFailure { device_id: String },

    #[error("invalid HMAC key reference: {0}")]
    InvalidHmacKeyRef(String),

    // ---- Hash chain ----
    #[error("hash chain broken at signal {signal_id}: expected {expected}, computed {computed}")]
    ChainBroken {
        signal_id: String,
        expected: String,
        computed: String,
    },

    #[error("hash chain is empty")]
    ChainEmpty,

    // ---- Monotonic counter ----
    #[error("monotonic counter out-of-order for device {device_id}: got {got}, expected > {prev}")]
    CounterOutOfOrder {
        device_id: String,
        got: i64,
        prev: i64,
    },

    // ---- Replay defence ----
    #[error(
        "replay detected: nonce already seen for device {device_id} within the 5-minute window"
    )]
    ReplayDetected { device_id: String },

    // ---- Device state ----
    #[error("device {device_id} is revoked: {reason}")]
    DeviceRevoked { device_id: String, reason: String },

    #[error("device {device_id} not found")]
    DeviceNotFound { device_id: String },

    #[error("session {session_id} not found")]
    SessionNotFound { session_id: String },

    // ---- Batch ----
    #[error("batch exceeds 1000-signal limit (got {count})")]
    BatchTooLarge { count: usize },

    // ---- Serialization ----
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    // ---- Database ----
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    // ---- ECC dispatch ----
    #[error("ECC dispatch failed: {0}")]
    EccDispatch(String),

    // ---- Snapshot / Merkle ----
    #[error("transparency log submission failed: {0}")]
    TlogSubmit(String),

    // ---- Generic ----
    #[error("internal error: {0}")]
    Internal(String),
}
