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

//! `eel-abis-enrolment`: enrolment lifecycle and 1:N deduplication pipeline.
//!
//! Drives biometric enrolment jobs from Received through quality check,
//! 1:N deduplication, adjudication routing, and final gallery placement.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod events;
pub mod orchestrator;
pub mod pipeline;
pub mod repository;

pub use domain::{
    CapturedTemplate, DedupDecision, DedupResult, EnrolmentDecision, EnrolmentJob,
    EnrolmentSource, EnrolmentStatus, FingerPosition, Modality, PadResult, ProbableMatch,
    QualityReport,
};
pub use error::EnrolmentError;
pub use pipeline::{apply_dedup_result, apply_quality_report, finalise, ingest, route_decision};
