//! Literal document types for a BVR enrolment workflow.

use serde::{Deserialize, Serialize};

/// Something a capture step can acquire.
///
/// Capture is deliberately modelled separately from biometric *matching*
/// modality. Two reasons:
///
/// - A capture need not produce a biometric template at all. A passport scan
///   produces MRZ text, and a liveness-proven photograph produces a face image,
///   so neither fits a "modality" enumeration.
/// - Palm and finger vein already exist as *matching* modalities in
///   `services/abis-svc/crates/abis-modality-vein` (ISO/IEC 19794-9), but the
///   capture trait in `sensor-driver` has no method for them, because NIR
///   vascular capture is a different sensor class from optical contact
///   capture. Conflating the two would imply a capture path that does not
///   exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureKind {
    /// Contact fingerprint image set, ISO/IEC 19794-2.
    Fingerprint,
    /// Near-infrared finger vascular pattern, ISO/IEC 19794-9.
    FingerVein,
    /// Near-infrared palm vascular pattern, ISO/IEC 19794-9.
    PalmVein,
    /// Near-infrared wrist vascular pattern, ISO/IEC 19794-9.
    WristVein,
    /// Face image, ISO/IEC 19794-5.
    Face,
    /// Face image with an active liveness challenge, ISO/IEC 30107-3 PAD.
    LivePhotograph,
    /// Iris image, ISO/IEC 19794-6.
    Iris,
    /// Optical document scan with an ICAO 9303 machine-readable zone.
    DocumentMrz,
}

impl CaptureKind {
    /// Returns `true` when this capture yields a biometric template or image
    /// subject to the biometric handling rules, as opposed to a document scan.
    pub fn is_biometric(self) -> bool {
        !matches!(self, CaptureKind::DocumentMrz)
    }
}

/// How many of a capture step's kinds must succeed.
///
/// This is what lets a workflow say "individually or in any combination"
/// without a combinatorial explosion of steps: one step lists the acceptable
/// captures and the policy says how many are needed. [`CapturePolicy::AtLeast`]
/// exists because a registration drive cannot always complete a full
/// acquisition on one citizen, and the platform already supports degraded
/// capture via `BiometricSensorDriver::capture_fingerprint_degraded`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapturePolicy {
    /// Every declared kind must be captured.
    AllOf,
    /// At least one declared kind must be captured.
    AnyOf,
    /// At least this many declared kinds must be captured.
    AtLeast(u16),
}

/// A named, reusable capture definition that steps reference by id.
///
/// Naming them separately from steps lets a workflow share one definition
/// across several steps, and lets a missing reference be caught at compile time
/// the same way a missing data item is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureRequirement {
    /// Stable identifier referenced by capture steps.
    pub id: String,
    /// Human-readable name, e.g. "Left palm and fingerprints".
    pub label: String,
    /// Acceptable capture kinds for this step.
    pub kinds: Vec<CaptureKind>,
    /// How many of them are needed.
    pub policy: CapturePolicy,
    /// Whether the step can be skipped entirely.
    #[serde(default)]
    pub optional: bool,
}

impl CaptureRequirement {
    /// Returns `true` when `captured` satisfies this requirement's policy.
    pub fn is_satisfied_by(&self, captured: &[CaptureKind]) -> bool {
        let present = |k: &CaptureKind| captured.contains(k);
        match self.policy {
            CapturePolicy::AllOf => self.kinds.iter().all(present),
            CapturePolicy::AnyOf => self.kinds.iter().any(present),
            CapturePolicy::AtLeast(n) => {
                self.kinds.iter().filter(|k| present(k)).count() as u16 >= n
            }
        }
    }

    /// Returns the declared kinds not present in `captured`.
    pub fn missing_from(&self, captured: &[CaptureKind]) -> Vec<CaptureKind> {
        self.kinds
            .iter()
            .copied()
            .filter(|k| !captured.contains(k))
            .collect()
    }
}

/// Widget and validator selected for a data item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataKind {
    /// Free text.
    Text,
    /// ISO 8601 date (e.g. `1990-06-15`).
    Date,
    /// Eight-digit national ID; pattern-validated.
    NationalId,
    /// Kenyan E.164 phone; pattern-validated.
    Phone,
    /// Polling station code; pattern-validated.
    StationCode,
    /// Closed choice from [`Constraints::options`].
    Choice,
    /// Explicit consent checkbox. Must be `true` to pass when required.
    Consent,
}

/// Value bounds and closed option set for a data item.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Constraints {
    /// Inclusive lower bound on string length.
    pub min_length: Option<usize>,
    /// Inclusive upper bound on string length.
    pub max_length: Option<usize>,
    /// Permitted values, required when the item's kind is [`DataKind::Choice`].
    pub options: Option<Vec<String>>,
    /// Inclusive minimum age in whole years, for date-of-birth items.
    pub min_age_years: Option<u16>,
    /// Inclusive maximum age in whole years, for date-of-birth items.
    pub max_age_years: Option<u16>,
}

impl Constraints {
    /// Returns `true` when this constraint set constrains nothing.
    pub fn is_empty(&self) -> bool {
        self.min_length.is_none()
            && self.max_length.is_none()
            && self.options.is_none()
            && self.min_age_years.is_none()
            && self.max_age_years.is_none()
    }
}

/// One collectable item in the enrolment form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataItem {
    /// Stable key; also the submitted field name.
    pub name: String,
    /// Widget and validator to use.
    pub kind: DataKind,
    /// Human-readable label shown to the enrolment officer.
    pub label: String,
    /// Whether the item must be satisfied before the step can advance.
    pub required: bool,
    /// PII tier, mirroring the contract's `x-eel-data-class` tag.
    pub data_class: String,
    /// Optional help text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Optional value bounds.
    #[serde(default, skip_serializing_if = "Constraints::is_empty")]
    pub constraints: Constraints,
}

impl DataItem {
    /// Returns `true` when the data class is one of the platform's five PII
    /// tiers, `A` through `E`, matching `PIIDataClass` in `eel-pii-rs`.
    ///
    /// An earlier version of this check accepted only A, B, and C because that
    /// is the subset the BVR contract tags on its `Voter` schema. The platform
    /// model is wider than the contract subset, so the check now follows
    /// `PIIDataClass`; the contract subset is a documentation concern, not a
    /// compilation rule.
    pub fn has_valid_data_class(&self) -> bool {
        matches!(self.data_class.as_str(), "A" | "B" | "C" | "D" | "E")
    }
}

/// What a step does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StepKind {
    /// Collect and validate the listed data items.
    DataEntry {
        /// Names of the data items this step edits.
        items: Vec<String>,
    },
    /// Acquire the named capture requirement.
    Capture {
        /// Id of the [`CaptureRequirement`] to perform.
        capture: String,
    },
    /// Present the accumulated record for officer confirmation.
    Review,
    /// Submit to the BVR enrolment endpoint.
    Submit,
}

/// One step in the enrolment flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// Stable identifier, unique within the workflow.
    pub id: String,
    /// What this step does.
    #[serde(flatten)]
    pub kind: StepKind,
    /// Optional label overriding the generated one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A complete enrolment workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowSpec {
    /// Spec schema version. Only `1` is understood.
    pub version: u32,
    /// Stable workflow identifier.
    pub id: String,
    /// Human-readable workflow name.
    pub label: String,
    /// Collectable data items.
    pub data_items: Vec<DataItem>,
    /// Named capture definitions. May be empty for a workflow that collects no
    /// biometrics, such as a document-only pre-registration review.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capture_requirements: Vec<CaptureRequirement>,
    /// Ordered steps.
    pub steps: Vec<Step>,
}
