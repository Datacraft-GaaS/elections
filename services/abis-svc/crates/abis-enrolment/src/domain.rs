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

//! Core domain types for the enrolment lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Source system that submitted this enrolment job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrolmentSource {
    BvrKit,
    LegacyImport,
    Diaspora,
    Rectification,
}

impl EnrolmentSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnrolmentSource::BvrKit => "BvrKit",
            EnrolmentSource::LegacyImport => "LegacyImport",
            EnrolmentSource::Diaspora => "Diaspora",
            EnrolmentSource::Rectification => "Rectification",
        }
    }
}

/// Life-cycle status of an enrolment job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrolmentStatus {
    Received,
    QualityChecking,
    DedupPending,
    DedupInProgress,
    DedupComplete,
    DedupHold,
    AdjudicationRequired,
    Enrolled,
    Rejected,
    Withdrawn,
}

impl EnrolmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnrolmentStatus::Received => "Received",
            EnrolmentStatus::QualityChecking => "QualityChecking",
            EnrolmentStatus::DedupPending => "DedupPending",
            EnrolmentStatus::DedupInProgress => "DedupInProgress",
            EnrolmentStatus::DedupComplete => "DedupComplete",
            EnrolmentStatus::DedupHold => "DedupHold",
            EnrolmentStatus::AdjudicationRequired => "AdjudicationRequired",
            EnrolmentStatus::Enrolled => "Enrolled",
            EnrolmentStatus::Rejected => "Rejected",
            EnrolmentStatus::Withdrawn => "Withdrawn",
        }
    }
}

/// Biometric modality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modality {
    Fingerprint,
    Face,
    Iris,
}

/// Finger position code (ISO/IEC 19794-2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FingerPosition {
    RightThumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightLittle,
    LeftThumb,
    LeftIndex,
    LeftMiddle,
    LeftRing,
    LeftLittle,
    Unknown,
}

/// A single captured biometric template with quality metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedTemplate {
    pub modality: Modality,
    pub finger_position: Option<FingerPosition>,
    /// NFIQ2 / ICAO / ISO 29794 quality score in [0.0, 1.0].
    pub quality_score: f32,
    /// BLAKE3 hash reference to the actual template bytes in the blob store.
    pub template_bytes_ref: String,
    /// Presentation attack detection result.
    pub pad_result: PadResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadResult {
    Live,
    Spoof,
    Unknown,
}

/// Aggregated quality assessment across all modalities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub per_modality: Vec<(Modality, f32)>,
    pub usable_finger_count: u8,
    pub iris_captured: bool,
    pub face_captured: bool,
    pub retake_reasons: Vec<String>,
}

impl QualityReport {
    /// True when the quality is sufficient to proceed with dedup.
    pub fn is_acceptable(&self) -> bool {
        self.retake_reasons.is_empty() && self.usable_finger_count >= 8 && self.face_captured
    }
}

/// An enrolment job submitted for processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrolmentJob {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub source: EnrolmentSource,
    pub captured_at: DateTime<Utc>,
    pub captured_by_kit_id: Option<String>,
    pub captured_by_officer: Option<String>,
    pub quality_report: Option<QualityReport>,
    pub templates: Vec<CapturedTemplate>,
    pub status: EnrolmentStatus,
}

impl EnrolmentJob {
    pub fn new(
        subject_id: Uuid,
        source: EnrolmentSource,
        captured_at: DateTime<Utc>,
        captured_by_kit_id: Option<String>,
        captured_by_officer: Option<String>,
        templates: Vec<CapturedTemplate>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            subject_id,
            source,
            captured_at,
            captured_by_kit_id,
            captured_by_officer,
            quality_report: None,
            templates,
            status: EnrolmentStatus::Received,
        }
    }
}

/// Outcome of a 1:N deduplication search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupResult {
    pub probable_matches: Vec<ProbableMatch>,
    pub decision: DedupDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbableMatch {
    pub subject_id: Uuid,
    pub score: f32,
    pub modality_contributions: Vec<(Modality, f32)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DedupDecision {
    NoMatch,
    HighConfidenceMatch,
    ManualAdjudicationRequired,
}

/// Final enrolment decision: written when a job reaches the Enrolled state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrolmentDecision {
    pub id: Uuid,
    pub enrolment_job_id: Uuid,
    pub subject_id: Uuid,
    pub enrolled_at: DateTime<Utc>,
    pub gallery_shard_id: String,
    pub previous_records: Vec<Uuid>,
    pub merge_from: Option<Uuid>,
    pub adjudicator_id: Option<Uuid>,
}
