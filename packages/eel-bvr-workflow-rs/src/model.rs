//! Literal document types for a BVR enrolment workflow.

use serde::{Deserialize, Serialize};

/// Biometric modality a capture step acquires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiometricModality {
    /// ISO/IEC 19794-2 fingerprint image set.
    Fingerprint,
    /// ISO/IEC 19794-5 face image.
    Face,
    /// ISO/IEC 19794-6 iris image.
    Iris,
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
    /// Returns `true` when the data class is one of the contract's A, B, or C.
    pub fn has_valid_data_class(&self) -> bool {
        matches!(self.data_class.as_str(), "A" | "B" | "C")
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
    /// Acquire biometric samples through the sensor driver abstraction.
    BiometricCapture {
        /// Modality to capture.
        modality: BiometricModality,
        /// Whether capture must succeed before advancing.
        required: bool,
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
    /// Ordered steps.
    pub steps: Vec<Step>,
}
