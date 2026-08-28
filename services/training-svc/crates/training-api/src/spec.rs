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
