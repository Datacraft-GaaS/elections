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
    eel_spec_helpers_rs::page_flat_value(items)
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
    State(state): State<SharedSpecState>,
    Path(assessment_id): Path<String>,
    Json(req): Json<spec::AssessmentSubmission>,
) -> axum::response::Response {
    info!(assessment_id, "submit_assessment");
    // The submission must belong to an enrolment that exists. Previously the
    // enrolment id was discarded (`let _ = (assessment_id, req)`).
    let known = state
        .enrolments
        .lock()
        .expect("lock poisoned")
        .contains_key(&req.enrolment_id);
    if !known {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "enrolment not found"})),
        )
            .into_response();
    }
    if req.answers.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "an assessment needs at least one answer"})),
        )
            .into_response();
    }

    // Score it against the question bank this crate serves at
    // `POST /v1/exams/questions`. A submitted answer is only right if it matches a
    // registered question's key, which is what stops a candidate scoring
    // themselves.
    let blueprint = blueprint();
    let bank: Vec<crate::exam::Question> = {
        let store = state.exam.store.read().await;
        store.questions.values().cloned().collect()
    };
    if bank.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "detail": "no assessment questions are registered, so a submission cannot be \
                           scored; register them at POST /v1/exams/questions first",
            })),
        )
            .into_response();
    }
    // The session's question set is the bank, so every question in it is marked.
    let session = crate::exam::ExamSession {
        session_id: assessment_id.clone(),
        application_id: req.enrolment_id.clone(),
        blueprint_id: blueprint.blueprint_id.clone(),
        attempt: 1,
        seed_hex: "eel.spec.assessment.seed.v1".to_string(),
        status: crate::exam::SessionStatus::Submitted,
        started_at: chrono::Utc::now(),
        deadline: chrono::Utc::now() + chrono::Duration::hours(3),
        question_ids: bank.iter().map(|q| q.question_id.clone()).collect(),
        receipt_hash_hex: None,
    };
    let answers = answer_indexes(&bank, &req.answers);
    let result = crate::exam::score_session(&session, &blueprint, &bank, &answers);
    let answered = req.answers.len();
    let correct = (result.total_pct / 100.0 * answered as f32).round() as usize;

    // A certificate is issued on a pass. It is not issued otherwise, and it is
    // recorded so `get_certificate` can find it.
    let certificate_id = if result.passed {
        let id = uuid::Uuid::now_v7().to_string();
        state
            .certificates
            .lock()
            .expect("lock poisoned")
            .insert(
                id.clone(),
                spec::CertificateRecord {
                    certificate_id: id.clone(),
                    enrolment_id: req.enrolment_id.clone(),
                    issued_at: chrono::Utc::now().to_rfc3339(),
                    expires_at: (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
                    // The document is rendered by the document service, which is
                    // not reachable from this process.
                    document_url: None,
                    blocking_reason: Some(
                        "the certificate document is rendered by the document service and \
                         signed by the board's HSM key; neither is reachable from this process"
                            .to_string(),
                    ),
                },
            );
        Some(id)
    } else {
        None
    };

    let record = spec::AssessmentSubmissionRecord {
        submission_id: uuid::Uuid::now_v7().to_string(),
        assessment_id: assessment_id.clone(),
        enrolment_id: req.enrolment_id.clone(),
        score_pct: f64::from(result.total_pct),
        passed: result.passed,
        correct,
        answered,
        certificate_id: certificate_id.clone(),
        assessed_at: chrono::Utc::now().to_rfc3339(),
        blocking_reason: (!result.passed).then(|| {
            format!(
                "the pass mark for this blueprint is {}%; the submission scored {:.1}%",
                blueprint.pass_mark_pct, result.total_pct
            )
        }),
    };
    state
        .submissions
        .lock()
        .expect("lock poisoned")
        .insert(record.submission_id.clone(), record.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "submission_id": record.submission_id,
            "assessment_id": assessment_id,
            "score_pct": record.score_pct,
            "passed": record.passed,
            "correct": record.correct,
            "answered": record.answered,
            "section_scores": result.section_scores,
            "certificate_id": record.certificate_id,
            "assessed_at": record.assessed_at,
            "blocking_reason": record.blocking_reason,
        })),
    )
        .into_response()
}

/// The blueprint the spec assessment surface scores against: the first seeded
/// per-role blueprint.
fn blueprint() -> crate::exam::ExamBlueprint {
    crate::exam::default_blueprints()
        .into_iter()
        .next()
        .expect("default_blueprints is not empty")
}

/// Map a candidate's submitted answers onto option indexes the scorer compares.
///
/// A submitted option that the bank does not offer maps to no index, which the
/// scorer counts as wrong. That is the point: a submission cannot score itself.
fn answer_indexes(
    bank: &[crate::exam::Question],
    answers: &[spec::AssessmentAnswer],
) -> std::collections::HashMap<String, Vec<usize>> {
    let mut out = std::collections::HashMap::new();
    for a in answers {
        let Some(q) = bank.iter().find(|q| q.question_id == a.question_id) else {
            continue;
        };
        // `selected_option` carries the option text; map it to its index.
        let idx: Vec<usize> = q
            .options
            .iter()
            .enumerate()
            .filter(|(_, opt)| *opt == &a.selected_option)
            .map(|(i, _)| i)
            .collect();
        out.insert(a.question_id.clone(), idx);
    }
    out
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
    State(state): State<SharedSpecState>,
    Path(certificate_id): Path<String>,
) -> axum::response::Response {
    info!(certificate_id, "get_certificate");
    // Previously any certificate id returned a URL and an `expires_at` of *now*,
    // so every certificate resolved and every certificate was already expired.
    let guard = state.certificates.lock().expect("lock poisoned");
    match guard.get(&certificate_id) {
        Some(c) => Json(serde_json::json!({
            "certificate_id": c.certificate_id,
            "enrolment_id": c.enrolment_id,
            "url": c.document_url,
            "issued_at": c.issued_at,
            "expires_at": c.expires_at,
            "blocking_reason": c.blocking_reason,
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "certificate not found"})),
        )
            .into_response(),
    }
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
            let approved = req.verdict == "approve";
            a.status = if approved { "approved" } else { "rejected" }.to_string();
            let application_id = a.application_id.clone();
            let organisation = a.organisation.clone();
            let application_type = a.application_type.clone();
            drop(guard);
            // An approval is what issues the badge `verify_badge` checks. A badge
            // number is derived from the application, so approving twice cannot
            // mint a second credential.
            let badge_number = if approved {
                // The registrar may assign the credential number on the decision.
                // Where they do not, one is derived from the application so that
                // an approved application always has something `verify_badge` can
                // check -- derived, so re-approving cannot mint a second one.
                let assigned = req
                    .badge_number
                    .as_deref()
                    .map(str::trim)
                    .filter(|n| !n.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                format!("EEL-{}", &uuid::Uuid::new_v5(
                    &uuid::Uuid::parse_str(&application_id).unwrap_or_else(|_| uuid::Uuid::now_v7()),
                    b"eel.training.accreditation.badge.v1",
                ).simple().to_string()[..12].to_uppercase())
                    });
                state.badges.lock().expect("lock poisoned").insert(
                    assigned.clone(),
                    spec::BadgeRecord {
                        badge_number: assigned.clone(),
                        application_id: application_id.clone(),
                        name_masked: mask_name(&organisation),
                        organisation: organisation.clone(),
                        r#type: application_type.clone(),
                        issued_at: chrono::Utc::now().to_rfc3339(),
                        valid_until: (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
                    },
                );
                Some(assigned)
            } else {
                None
            };
            Json(serde_json::json!({
                "decision_id": uuid::Uuid::now_v7().to_string(),
                "decided_at": chrono::Utc::now().to_rfc3339(),
                "verdict": req.verdict,
                "badge_number": badge_number,
            })).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "application not found"}))).into_response(),
    }
}

pub async fn verify_badge(
    State(state): State<SharedSpecState>,
    Path(badge_number): Path<String>,
) -> axum::response::Response {
    info!(badge_number, "verify_badge");
    // This returned `valid: true` for **any** badge number. It is the endpoint a
    // verifier trusts to check an accreditation credential, and it accepted every
    // string presented to it.
    let guard = state.badges.lock().expect("lock poisoned");
    let Some(badge) = guard.get(&badge_number).cloned() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "detail": format!("no accreditation badge with number {badge_number}"),
            })),
        )
            .into_response();
    };
    let now = chrono::Utc::now();
    let valid_until = chrono::DateTime::parse_from_rfc3339(&badge.valid_until)
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or(now);
    Json(spec::BadgeVerification {
        // A badge past its validity is not valid. That is the whole question this
        // endpoint answers.
        valid: valid_until > now,
        name_masked: Some(badge.name_masked),
        organisation: badge.organisation,
        r#type: badge.r#type,
        valid_until: Some(badge.valid_until),
    })
    .into_response()
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

/// Mask an organisation name for display on a badge.
///
/// A badge is read by a verifier, so the holder's name is shown masked rather
/// than in full.
fn mask_name(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    match chars.len() {
        0 | 1 => "*".to_string(),
        n => format!("{}{}{}", chars[0], "*".repeat(n.saturating_sub(2).min(8)), chars[n - 1]),
    }
}

pub async fn publish_content(
    State(state): State<SharedSpecState>,
    Json(req): Json<spec::PublishContentRequest>,
) -> axum::response::Response {
    info!("publish_content");
    if req.title.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "title is required"})),
        )
            .into_response();
    }
    // Previously the content was echoed back inside the response and stored
    // nowhere, so `list_content` was always empty and `get_content` returned a
    // bare id for anything.
    let now = chrono::Utc::now().to_rfc3339();
    let status = req.status.unwrap_or_else(|| "published".to_string());
    let record = spec::ContentRecord {
        content_id: uuid::Uuid::now_v7().to_string(),
        title: req.title,
        body: req.body,
        published_at: Some(now.clone()),
        updated_at: now,
        status,
        version: 1,
        translations: Default::default(),
    };
    state
        .content
        .lock()
        .expect("lock poisoned")
        .insert(record.content_id.clone(), record.clone());
    (StatusCode::CREATED, Json(record)).into_response()
}

pub async fn list_content(State(state): State<SharedSpecState>) -> impl IntoResponse {
    info!("list_content");
    let guard = state.content.lock().expect("lock poisoned");
    let mut items: Vec<spec::ContentRecord> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.content_id.cmp(&b.content_id));
    Json(page(&serde_json::to_value(items).unwrap()))
}

pub async fn get_content(
    State(state): State<SharedSpecState>,
    Path(content_id): Path<String>,
) -> axum::response::Response {
    info!(content_id, "get_content");
    let guard = state.content.lock().expect("lock poisoned");
    match guard.get(&content_id) {
        Some(c) => Json(c.clone()).into_response(),
        // Previously returned `{ "content_id": <id> }` with 200 for any id.
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "content not found"})),
        )
            .into_response(),
    }
}

pub async fn update_content(
    State(state): State<SharedSpecState>,
    Path(content_id): Path<String>,
    Json(req): Json<spec::PublishContentRequest>,
) -> axum::response::Response {
    info!(content_id, "update_content");
    let mut guard = state.content.lock().expect("lock poisoned");
    let Some(existing) = guard.get_mut(&content_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "content not found"})),
        )
            .into_response();
    };
    if req.title.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "title is required"})),
        )
            .into_response();
    }
    // Every update bumps the version, so a reader can tell two revisions apart.
    // Previously the content was echoed back and the old version lost.
    existing.title = req.title;
    existing.body = req.body;
    if let Some(status) = req.status {
        existing.status = status;
    }
    existing.version += 1;
    existing.updated_at = chrono::Utc::now().to_rfc3339();
    let updated = existing.clone();
    drop(guard);
    Json(updated).into_response()
}

pub async fn delete_content(
    State(state): State<SharedSpecState>,
    Path(content_id): Path<String>,
) -> axum::response::Response {
    info!(content_id, "delete_content");
    // Previously answered 204 for any id, so deleting content that did not exist
    // reported success.
    let removed = state
        .content
        .lock()
        .expect("lock poisoned")
        .remove(&content_id);
    if removed.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "content not found"})),
        )
            .into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

pub async fn add_content_translation(
    State(state): State<SharedSpecState>,
    Path(content_id): Path<String>,
    Json(req): Json<spec::ContentTranslationRequest>,
) -> axum::response::Response {
    info!(content_id, "add_content_translation");
    let tag = req.language.trim();
    if tag.is_empty() || !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "language must be a BCP-47 tag"})),
        )
            .into_response();
    }
    let mut guard = state.content.lock().expect("lock poisoned");
    let Some(existing) = guard.get_mut(&content_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "content not found"})),
        )
            .into_response();
    };
    // Previously the translation was echoed back and dropped, so a content item
    // could accumulate no translations at all.
    if existing.translations.contains_key(tag) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": format!("a {tag} translation already exists")})),
        )
            .into_response();
    }
    existing.translations.insert(
        tag.to_string(),
        serde_json::json!({ "title": req.title, "body": req.body }),
    );
    existing.version += 1;
    existing.updated_at = chrono::Utc::now().to_rfc3339();
    let updated = existing.clone();
    drop(guard);
    (StatusCode::CREATED, Json(updated)).into_response()
}

/// Composes the public training contract router with its own state.
pub fn router(state: SharedSpecState) -> axum::Router {
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
        .with_state(state)
}
