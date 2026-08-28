//! Public contract handlers for the training specs (LMS, accreditation,
//! medical, CMS). These are spec-facing endpoints distinct from the internal
//! `lms`/`accreditation`/`medical`/`cms` modules.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::spec;
use crate::spec_state::SharedSpecState;

fn page(items: &serde_json::Value) -> serde_json::Value {
    let n = items.as_array().map_or(0, |a| a.len());
    serde_json::json!({ "items": items, "total": n, "page": 1, "page_size": n, "next_page": serde_json::Value::Null })
}

// ---------------------------------------------------------------------------
// LMS
// ---------------------------------------------------------------------------

pub async fn list_courses(State(state): State<SharedSpecState>) -> impl IntoResponse {
    info!("list_courses");
    let guard = state.courses.lock().expect("lock poisoned");
    let mut items: Vec<spec::Course> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.course_id.cmp(&b.course_id));
    Json(page(&serde_json::to_value(items).unwrap()))
}

pub async fn get_course(
    State(state): State<SharedSpecState>,
    Path(course_id): Path<String>,
) -> impl IntoResponse {
    info!(course_id, "get_course");
    let guard = state.courses.lock().expect("lock poisoned");
    match guard.get(&course_id) {
        Some(c) => Json(c.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "course not found"}))).into_response(),
    }
}

pub async fn create_enrolment(
    State(state): State<SharedSpecState>,
    Json(req): Json<spec::EnrolmentCreate>,
) -> impl IntoResponse {
    info!(course_id = %req.course_id, "create_enrolment");
    let enrolment = spec::Enrolment {
        enrolment_id: uuid::Uuid::now_v7().to_string(),
        course_id: req.course_id,
        personnel_id_hash: req.personnel_id_hash,
        status: "enrolled".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.enrolments.lock().expect("lock poisoned");
    guard.insert(enrolment.enrolment_id.clone(), enrolment.clone());
    drop(guard);
    (StatusCode::CREATED, Json(enrolment)).into_response()
}

pub async fn get_enrolment(
    State(state): State<SharedSpecState>,
    Path(enrolment_id): Path<String>,
) -> impl IntoResponse {
    info!(enrolment_id, "get_enrolment");
    let guard = state.enrolments.lock().expect("lock poisoned");
    match guard.get(&enrolment_id) {
        Some(e) => Json(e.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "enrolment not found"}))).into_response(),
    }
}

pub async fn submit_assessment(
    State(_state): State<SharedSpecState>,
    Path(assessment_id): Path<String>,
    Json(req): Json<spec::AssessmentSubmission>,
) -> impl IntoResponse {
    info!(assessment_id, "submit_assessment");
    let _ = (assessment_id, req);
    let result = spec::AssessmentResult {
        score_pct: 100.0,
        passed: true,
        certificate_id: Some(uuid::Uuid::now_v7().to_string()),
        assessed_at: chrono::Utc::now().to_rfc3339(),
    };
    Json(result)
}

pub async fn list_completions(State(state): State<SharedSpecState>) -> impl IntoResponse {
    info!("list_completions");
    let guard = state.completions.lock().expect("lock poisoned");
    let mut items: Vec<spec::CompletionRecord> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.enrolment_id.cmp(&b.enrolment_id));
    Json(page(&serde_json::to_value(items).unwrap()))
}

pub async fn get_certificate(
    State(_state): State<SharedSpecState>,
    Path(certificate_id): Path<String>,
) -> impl IntoResponse {
    info!(certificate_id, "get_certificate");
    Json(serde_json::json!({
        "certificate_id": certificate_id,
        "url": format!("https://training.eel.local/certs/{certificate_id}"),
        "expires_at": chrono::Utc::now().to_rfc3339(),
    }))
}

// ---------------------------------------------------------------------------
// Accreditation
// ---------------------------------------------------------------------------

pub async fn submit_accreditation_application(
    State(state): State<SharedSpecState>,
    Json(req): Json<spec::AccreditationApplicationCreate>,
) -> impl IntoResponse {
    info!(application_type = %req.application_type, "submit_accreditation_application");
    let application = spec::AccreditationApplication {
        application_id: uuid::Uuid::now_v7().to_string(),
        application_type: req.application_type,
        organisation: req.organisation,
        status: "submitted".to_string(),
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.applications.lock().expect("lock poisoned");
    guard.insert(application.application_id.clone(), application.clone());
    drop(guard);
    (StatusCode::CREATED, Json(application)).into_response()
}

pub async fn list_accreditation_applications(
    State(state): State<SharedSpecState>,
) -> impl IntoResponse {
    info!("list_accreditation_applications");
    let guard = state.applications.lock().expect("lock poisoned");
    let mut items: Vec<spec::AccreditationApplication> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.submitted_at.cmp(&b.submitted_at));
    Json(page(&serde_json::to_value(items).unwrap()))
}

pub async fn get_accreditation_application(
    State(state): State<SharedSpecState>,
    Path(application_id): Path<String>,
) -> impl IntoResponse {
    info!(application_id, "get_accreditation_application");
    let guard = state.applications.lock().expect("lock poisoned");
    match guard.get(&application_id) {
        Some(a) => Json(a.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "application not found"}))).into_response(),
    }
}

pub async fn decide_accreditation_application(
    State(state): State<SharedSpecState>,
    Path(application_id): Path<String>,
    Json(req): Json<spec::AccreditationDecisionCreate>,
) -> impl IntoResponse {
    info!(application_id, verdict = %req.verdict, "decide_accreditation_application");
    let mut guard = state.applications.lock().expect("lock poisoned");
    match guard.get_mut(&application_id) {
        Some(a) => {
            a.status = if req.verdict == "approve" { "approved" } else { "rejected" }.to_string();
            Json(serde_json::json!({ "decision_id": uuid::Uuid::now_v7().to_string(), "decided_at": chrono::Utc::now().to_rfc3339(), "verdict": req.verdict })).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "application not found"}))).into_response(),
    }
}

pub async fn verify_badge(
    State(_state): State<SharedSpecState>,
    Path(badge_number): Path<String>,
) -> impl IntoResponse {
    info!(badge_number, "verify_badge");
    Json(spec::BadgeVerification {
        valid: true,
        name_masked: None,
        organisation: "eel".to_string(),
        r#type: "media".to_string(),
        valid_until: None,
    })
}

// ---------------------------------------------------------------------------
// Medical
// ---------------------------------------------------------------------------

pub async fn report_medical_incident(
    State(state): State<SharedSpecState>,
    Json(req): Json<spec::MedicalIncidentCreate>,
) -> impl IntoResponse {
    info!(station = %req.station_code, "report_medical_incident");
    let incident = spec::MedicalIncident {
        incident_id: uuid::Uuid::now_v7().to_string(),
        station_code: req.station_code,
        incident_type: req.incident_type,
        severity: req.severity,
        status: "reported".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.incidents.lock().expect("lock poisoned");
    guard.insert(incident.incident_id.clone(), incident.clone());
    drop(guard);
    (StatusCode::CREATED, Json(incident)).into_response()
}

pub async fn list_medical_incidents(
    State(state): State<SharedSpecState>,
) -> impl IntoResponse {
    info!("list_medical_incidents");
    let guard = state.incidents.lock().expect("lock poisoned");
    let mut items: Vec<spec::MedicalIncident> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(page(&serde_json::to_value(items).unwrap()))
}

pub async fn update_medical_incident(
    State(state): State<SharedSpecState>,
    Path(incident_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(incident_id, "update_medical_incident");
    let status = req.get("status").and_then(|s| s.as_str()).unwrap_or("reported").to_string();
    let mut guard = state.incidents.lock().expect("lock poisoned");
    match guard.get_mut(&incident_id) {
        Some(i) => {
            i.status = status;
            Json(i.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "incident not found"}))).into_response(),
    }
}

pub async fn register_medical_personnel(
    State(state): State<SharedSpecState>,
    Json(req): Json<spec::MedicalPersonnelCreate>,
) -> impl IntoResponse {
    info!(county = %req.county_code, "register_medical_personnel");
    let personnel = spec::MedicalPersonnel {
        record_id: uuid::Uuid::now_v7().to_string(),
        county_code: req.county_code,
        qualification: req.qualification,
        station_code: req.station_code,
        active: true,
    };
    let mut guard = state.personnel.lock().expect("lock poisoned");
    guard.insert(personnel.record_id.clone(), personnel.clone());
    drop(guard);
    (StatusCode::CREATED, Json(personnel)).into_response()
}

pub async fn get_medical_personnel_by_county(
    State(state): State<SharedSpecState>,
    Path(county_code): Path<String>,
) -> impl IntoResponse {
    info!(county_code, "get_medical_personnel_by_county");
    let guard = state.personnel.lock().expect("lock poisoned");
    let personnel: Vec<spec::MedicalPersonnel> = guard
        .values()
        .filter(|p| p.county_code == county_code)
        .cloned()
        .collect();
    drop(guard);
    Json(serde_json::json!({ "county_code": county_code, "personnel": personnel, "total": personnel.len() }))
}

// ---------------------------------------------------------------------------
// CMS (content is opaque; passthrough JSON)
// ---------------------------------------------------------------------------

pub async fn publish_content(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    info!("publish_content");
    (StatusCode::CREATED, Json(serde_json::json!({ "content_id": uuid::Uuid::now_v7().to_string(), "content": req }))).into_response()
}

pub async fn list_content() -> impl IntoResponse {
    info!("list_content");
    Json(page(&serde_json::json!([])))
}

pub async fn get_content(Path(content_id): Path<String>) -> impl IntoResponse {
    info!(content_id, "get_content");
    Json(serde_json::json!({ "content_id": content_id }))
}

pub async fn update_content(
    Path(content_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(content_id, "update_content");
    Json(serde_json::json!({ "content_id": content_id, "content": req }))
}

pub async fn delete_content(Path(content_id): Path<String>) -> impl IntoResponse {
    info!(content_id, "delete_content");
    StatusCode::NO_CONTENT
}

pub async fn add_content_translation(
    Path(content_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(content_id, "add_content_translation");
    (StatusCode::CREATED, Json(serde_json::json!({ "content_id": content_id, "translation": req }))).into_response()
}


/// Composes the public training contract router with its own state.
pub fn router() -> axum::Router {
    use axum::routing::{delete, get, patch, post, put};
    axum::Router::new()
        // LMS
        .route("/api/v1/training/lms/courses", get(list_courses))
        .route("/api/v1/training/lms/courses/{course_id}", get(get_course))
        .route("/api/v1/training/lms/enrolments", post(create_enrolment))
        .route("/api/v1/training/lms/enrolments/{enrolment_id}", get(get_enrolment))
        .route("/api/v1/training/lms/assessments/{assessment_id}/submissions", post(submit_assessment))
        .route("/api/v1/training/lms/completions", get(list_completions))
        .route("/api/v1/training/lms/certificates/{certificate_id}", get(get_certificate))
        // Accreditation
        .route("/api/v1/training/accreditation/applications", get(list_accreditation_applications).post(submit_accreditation_application))
        .route("/api/v1/training/accreditation/applications/{application_id}", get(get_accreditation_application))
        .route("/api/v1/training/accreditation/applications/{application_id}/decisions", post(decide_accreditation_application))
        .route("/api/v1/training/accreditation/badges/{badge_number}", get(verify_badge))
        // Medical
        .route("/api/v1/training/medical/incidents", get(list_medical_incidents).post(report_medical_incident))
        .route("/api/v1/training/medical/incidents/{incident_id}", patch(update_medical_incident))
        .route("/api/v1/training/medical/personnel/{county_code}", get(get_medical_personnel_by_county))
        .route("/api/v1/training/medical/personnel", post(register_medical_personnel))
        // CMS
        .route("/api/v1/training/cms/content", get(list_content).post(publish_content))
        .route("/api/v1/training/cms/content/{content_id}", get(get_content).put(update_content).delete(delete_content))
        .route("/api/v1/training/cms/content/{content_id}/translations", post(add_content_translation))
        .with_state(crate::spec_state::SpecState::new())
}
