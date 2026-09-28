//! Per-datum provenance: where, when, who, why, and how.
//!
//! This is the platform's five-axis audit reference, the same one the rest of
//! the codebase preserves (`where` / `when` / `who` / `why` / `how`) alongside a
//! `proofHash`, a `signature`, and a UUIDv7 `auditId`. It is recorded per record
//! rather than per batch so a single extracted datum is still traceable.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Why a datum was collected. Mirrors `eel-pii-rs::Purpose` for the cases BVR
/// actually produces, so a disclosure can be audited against a declared purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    /// Enrolling a new voter or updating an existing registration.
    Enrolment,
    /// Adjudicating a registration dispute or appeal.
    Adjudication,
    /// Producing a statutory report required by law.
    StatutoryReport,
    /// Receiving or processing a grievance or petition.
    GrievanceIntake,
}

/// How the value was obtained, which determines how much it can be trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMethod {
    /// Typed by the enrolment officer from an identity document.
    OfficerTranscription,
    /// Read automatically from a scanned document's machine-readable zone.
    DocumentMrz,
    /// Acquired from a biometric sensor through the driver abstraction.
    BiometricSensor,
    /// Taken by the system's own cross-check against another source.
    DerivedFromOtherRecords,
}

/// The five-axis audit reference attached to a record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// UUIDv7 audit identifier.
    pub audit_id: Uuid,
    /// **where**: the instance that captured it.
    pub instance_id: Uuid,
    /// **where**: the human-readable station label at capture time.
    pub station_label: String,
    /// **when**: capture instant, UTC.
    pub captured_at: DateTime<Utc>,
    /// **who**: the enrolment officer's identity.
    pub operator_id: String,
    /// **why**: the declared purpose.
    pub purpose: Purpose,
    /// **how**: the workflow that produced it.
    pub workflow_id: String,
    /// **how**: the step within that workflow.
    pub step_id: String,
    /// **how**: how the value itself was obtained.
    pub capture_method: CaptureMethod,
}

impl Provenance {
    /// Builds a provenance block with a fresh `audit_id`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_id: Uuid,
        station_label: &str,
        operator_id: &str,
        purpose: Purpose,
        workflow_id: &str,
        step_id: &str,
        capture_method: CaptureMethod,
    ) -> Self {
        Self {
            audit_id: Uuid::now_v7(),
            instance_id,
            station_label: station_label.to_owned(),
            captured_at: Utc::now(),
            operator_id: operator_id.to_owned(),
            purpose,
            workflow_id: workflow_id.to_owned(),
            step_id: step_id.to_owned(),
            capture_method,
        }
    }
}
