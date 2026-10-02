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

//! `eel-inventory-rfid-cover-custody`
//!
//! Tamper-evident RFID cover custody for ballot-box covers.
//! Implements: tag issuance, pairing, read ingest, FSM transitions,
//! BLAKE3 hash chain, tamper alerts, replay defence, and Prometheus metrics.
//!
//! Ref: docs/features/rfid-ballot-box-covers.md
#![warn(missing_docs)]

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
