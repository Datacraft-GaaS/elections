//! Public contract models for the training specs (LMS, accreditation, medical, CMS).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// LMS (training-lms/lms.yaml)
// ---------------------------------------------------------------------------

/// Public course view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub course_id: String,
    pub title_en: String,
    pub title_sw: String,
    pub description_en: Option<String>,
    pub description_sw: Option<String>,
    pub duration_minutes: u32,
    pub passing_score_pct: u8,
    pub role_target: Option<String>,
    pub mandatory: bool,
    pub created_at: String,
}

/// Request body for creating an enrolment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrolmentCreate {
    pub personnel_id_hash: String,
    pub course_id: String,
}

/// Public enrolment view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrolment {
    pub enrolment_id: String,
    pub course_id: String,
    pub personnel_id_hash: String,
    pub status: String,
    pub created_at: String,
}

/// Request body for submitting an assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSubmission {
    pub enrolment_id: String,
    pub answers: Vec<AssessmentAnswer>,
}

/// One answer in an assessment submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentAnswer {
    pub question_id: String,
    pub selected_option: String,
}

/// Public assessment result view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub score_pct: f64,
    pub passed: bool,
    pub certificate_id: Option<String>,
    pub assessed_at: String,
}

/// Public completion record view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRecord {
    pub enrolment_id: String,
    pub course_id: String,
    pub county_code: String,
    pub status: String,
    pub completed_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Accreditation (training-accreditation/accreditation.yaml)
// ---------------------------------------------------------------------------

/// Request body for submitting an accreditation application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccreditationApplicationCreate {
    pub applicant_id_hash: String,
    pub application_type: String,
    pub organisation: String,
    pub id_document_hash: String,
    pub photo_hash: String,
    pub supporting_docs_refs: Vec<String>,
}

/// Public accreditation application view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccreditationApplication {
    pub application_id: String,
    pub application_type: String,
    pub organisation: String,
    pub status: String,
    pub submitted_at: String,
}

/// Request body for an accreditation decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccreditationDecisionCreate {
    pub verdict: String,
    pub grounds: String,
    pub badge_number: Option<String>,
}

/// Public badge verification view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeVerification {
    pub valid: bool,
    pub name_masked: Option<String>,
    pub organisation: String,
    pub r#type: String,
    pub valid_until: Option<String>,
}

// ---------------------------------------------------------------------------
// Medical (training-medical/medical.yaml)
// ---------------------------------------------------------------------------

/// Request body for reporting a medical incident.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalIncidentCreate {
    pub station_code: String,
    pub incident_type: String,
    pub severity: String,
    pub first_responder_id_hash: String,
    pub reported_at: String,
    pub notes: String,
}

/// Public medical incident view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalIncident {
    pub incident_id: String,
    pub station_code: String,
    pub incident_type: String,
    pub severity: String,
    pub status: String,
    pub created_at: String,
}

/// Request body for registering medical personnel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalPersonnelCreate {
    pub personnel_id_hash: String,
    pub county_code: String,
    pub qualification: String,
    pub station_code: String,
    pub phone_hash: String,
}

/// Public medical personnel view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalPersonnel {
    pub record_id: String,
    pub county_code: String,
    pub qualification: String,
    pub station_code: String,
    pub active: bool,
}

// ---------------------------------------------------------------------------
// Records the spec surface issues rather than inventing
// ---------------------------------------------------------------------------

/// A scored assessment submission.
///
/// `submit_assessment` previously returned `score_pct: 100.0, passed: true` for
/// **every** submission and minted a certificate id, so every candidate passed
/// every assessment. The score is now computed by the crate's own
/// `exam::score_session`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSubmissionRecord {
    pub submission_id: String,
    pub assessment_id: String,
    pub enrolment_id: String,
    pub score_pct: f64,
    pub passed: bool,
    /// Questions the candidate got right, out of those submitted.
    pub correct: usize,
    pub answered: usize,
    pub certificate_id: Option<String>,
    pub assessed_at: String,
    /// Why no certificate was issued, when none was.
    pub blocking_reason: Option<String>,
}

/// An issued certificate.
///
/// `get_certificate` previously returned a URL for any `certificate_id` and an
/// `expires_at` of *now*, so every certificate was instantly expired and every id
/// resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRecord {
    pub certificate_id: String,
    pub enrolment_id: String,
    pub issued_at: String,
    pub expires_at: String,
    /// Where the certificate document lives, once rendered.
    pub document_url: Option<String>,
    /// Certificates are not signed here: the board's signing key is in the HSM.
    pub blocking_reason: Option<String>,
}

/// An issued accreditation badge.
///
/// `verify_badge` previously returned `valid: true` for **any** badge number, so
/// the endpoint that a verifier trusts to check a journalist's accreditation
/// credential accepted every string presented to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeRecord {
    pub badge_number: String,
    pub application_id: String,
    /// The holder's name, masked for display.
    pub name_masked: String,
    pub organisation: String,
    pub r#type: String,
    pub issued_at: String,
    pub valid_until: String,
}

/// A published CMS content item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRecord {
    pub content_id: String,
    pub title: String,
    pub body: serde_json::Value,
    pub status: String,
    pub version: u32,
    pub published_at: Option<String>,
    pub updated_at: String,
    /// Translations, keyed by BCP-47 tag.
    pub translations: std::collections::BTreeMap<String, serde_json::Value>,
}

/// A request to publish CMS content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishContentRequest {
    pub title: String,
    #[serde(default)]
    pub body: serde_json::Value,
    #[serde(default)]
    pub status: Option<String>,
}

/// A request to add a translation to a content item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTranslationRequest {
    /// BCP-47 tag, e.g. "sw".
    pub language: String,
    #[serde(default)]
    pub title: Option<String>,
    pub body: serde_json::Value,
}
