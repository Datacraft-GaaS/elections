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

//! `eel-abis-bulk-ops`: batch biometric operations with idempotent resumption.
//!
//! Supports 7 operation types: enrol, dedup, delete, export, rewrap, import, retire.
//! Each operation is resumable from a progress checkpoint. Failure manifests are
//! written as CSV.
#![warn(missing_docs)]

pub mod api;
pub mod dedup;
pub mod delete;
pub mod dispatcher;
pub mod domain;
pub mod enrol;
pub mod error;
pub mod export;
pub mod import;
pub mod progress;
pub mod retire;
pub mod rewrap;

pub use domain::{BulkOperation, BulkOpState, BulkOpType, FailureRecord, ProgressSnapshot};
pub use error::BulkOpsError;
