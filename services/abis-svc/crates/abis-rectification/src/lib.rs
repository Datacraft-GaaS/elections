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

//! `eel-abis-rectification`: subject-driven biometric correction workflow.
//!
//! Requests flow through: submission -> triage -> evidence_pending -> approved/rejected -> completed.
//! SLA is 30 days from submission. SLA sweep runs every 15 minutes.
#![warn(missing_docs)]

pub mod api;
pub mod domain;
pub mod error;
pub mod policy;
pub mod scheduler;
pub mod workflow;

pub use domain::{EvidenceType, RectificationRequest, RectificationRequestType, RectificationState, SlaStatus};
pub use error::RectificationError;
