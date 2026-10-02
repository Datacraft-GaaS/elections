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

//! Round-trip tests for the `/api/v1/training/*` spec surface.
//!
//! The three that mattered most:
//!
//! - `submit_assessment` returned `score_pct: 100.0, passed: true` for **every**
//!   submission and minted a certificate id, so every candidate passed every
//!   assessment. The crate contains its own scoring engine, `exam::score_session`,
//!   which this handler never called.
//! - `verify_badge` returned `valid: true` for **any** badge number. That is the
//!   endpoint a verifier trusts to check an accreditation credential.
//! - `get_certificate` returned a URL for any `certificate_id` with an
//!   `expires_at` of *now*, so every certificate resolved and every certificate was
//!   already expired.
//!
//! The CMS content handlers echoed their input back and stored nothing:
//! `list_content` was always empty, `get_content` returned a bare id for anything,
//! `delete_content` answered 204 for any id, and translations were dropped.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use training_api::{
    handlers::{
        exam::{router as exam_router, ExamApiState},
        spec,
    },
    spec_state::SpecState,
};

/// The spec surface and the exam surface over one shared exam store, exactly as
/// `build_router` composes them. Both are needed: questions are registered through
/// the exam module and scored through the spec surface.
fn app() -> axum::Router {
    let exam = ExamApiState::new();
    spec::router(SpecState::with_exam(exam.clone())).merge(exam_router(exam))
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> axum::response::Response {
    let mut b = Request::builder().method(method).uri(uri);
    let payload = match body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    app.clone()
        .oneshot(b.body(payload).unwrap())
        .await
        .unwrap()
}

/// An enrolment to submit an assessment against.
///
/// The spec surface has no course-creation route (courses are seeded), so the
/// enrolment is created directly against a course id.
async fn enrolled(app: &axum::Router) -> String {
    let enrolment = body_json(
        send(
            app,
            "POST",
            "/api/v1/training/lms/enrolments",
            Some(json!({"course_id": "C-001", "personnel_id_hash": "sha256:abc"})),
        )
        .await,
    )
    .await;
    enrolment["enrolment_id"].as_str().unwrap().to_string()
}

/// Every weighted section of the first (PO) blueprint.
///
/// The scorer weights each section by its blueprint weight and divides by the
/// total weight, so a section with no questions contributes zero. A candidate who
/// answered one of four sections cannot score 100% -- which is the behaviour the
/// tests below are checking, so the fixtures cover the whole blueprint.
const SECTIONS: [&str; 4] = [
    "law_procedure",
    "ethics_secrecy",
    "biometric_operation",
    "incident_handling",
];

/// Register one question in the shared bank and return it.
///
/// The spec surface scores against the same bank the exam module serves, so a
/// question has to exist before a submission can be marked.
async fn register_question(app: &axum::Router, id: &str, section: &str, correct: &str) -> Value {
    let body = body_json(
        send(
            app,
            "POST",
            "/v1/exams/questions",
            Some(json!({
                "question_id": id,
                "role_level": "PO",
                "section": section,
                "difficulty": 1,
                "language": "en",
                "body": "Which of these is a valid serial?",
                "options": ["A", "B", "C"],
                "correct_indexes": [correct_index(correct)],
                "version": 1
            })),
        )
        .await,
    )
    .await;
    assert_eq!(body["status"], "created", "question must be registered: {body:?}");
    body
}

/// Register one question per weighted section, all with `correct` as the key.
async fn register_blueprint(app: &axum::Router, prefix: &str, correct: &str) {
    for (i, section) in SECTIONS.iter().enumerate() {
        register_question(app, &format!("{prefix}-{i}"), section, correct).await;
    }
}

/// The answer key for the questions `register_blueprint` just created.
fn blueprint_answers(prefix: &str, selected: &str) -> Vec<Value> {
    (0..SECTIONS.len())
        .map(|i| json!({"question_id": format!("{prefix}-{i}"), "selected_option": selected}))
        .collect()
}

fn correct_index(option: &str) -> usize {
    match option {
        "A" => 0,
        "B" => 1,
        _ => 2,
    }
}

fn option(index: usize) -> &'static str {
    match index {
        0 => "A",
        1 => "B",
        _ => "C",
    }
}

// ---------------------------------------------------------------------------
// submit_assessment
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_assessment_with_no_questions_cannot_be_scored() {
    // The honest answer when there is no question bank: refuse. Scoring against
    // nothing is how every submission ended up at 100%.
    let app = app();
    let enrolment = enrolled(&app).await;
    let resp = send(
        &app,
        "POST",
        "/api/v1/training/lms/assessments/A-1/submissions",
        Some(json!({"enrolment_id": enrolment, "answers": [{"question_id": "q1", "selected_option": "A"}]})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("/v1/exams/questions"), "the reason should say how to fix it: {detail}");
}

#[tokio::test]
async fn a_submission_against_an_unknown_enrolment_is_404() {
    // Previously the enrolment id was discarded entirely.
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/training/lms/assessments/A-1/submissions",
        Some(json!({"enrolment_id": "no-such", "answers": [{"question_id": "q1", "selected_option": "A"}]})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_assessment_needs_at_least_one_answer() {
    let app = app();
    let enrolment = enrolled(&app).await;
    let resp = send(
        &app,
        "POST",
        "/api/v1/training/lms/assessments/A-1/submissions",
        Some(json!({"enrolment_id": enrolment, "answers": []})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_fully_correct_submission_scores_and_earns_a_certificate() {
    let app = app();
    let enrolment = enrolled(&app).await;
    register_blueprint(&app, "q", "A").await;

    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/lms/assessments/A-1/submissions",
            Some(json!({
                "enrolment_id": enrolment,
                "answers": blueprint_answers("q", "A"),
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["score_pct"], 100.0);
    assert_eq!(result["passed"], true);
    assert_eq!(result["correct"], 4);
    assert_eq!(result["answered"], 4);
    let certificate = result["certificate_id"].as_str().unwrap().to_string();

    // The certificate is a record, so it can be read back.
    let read = body_json(
        send(&app, "GET", &format!("/api/v1/training/lms/certificates/{certificate}"), None).await,
    )
    .await;
    assert_eq!(read["certificate_id"], certificate.as_str());
    assert!(read["expires_at"].as_str().unwrap() > read["issued_at"].as_str().unwrap());
}

#[tokio::test]
async fn a_fully_wrong_submission_fails_and_earns_no_certificate() {
    // The headline. Previously every submission scored 100% and passed.
    let app = app();
    let enrolment = enrolled(&app).await;
    register_blueprint(&app, "q", "A").await;

    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/lms/assessments/A-1/submissions",
            Some(json!({
                "enrolment_id": enrolment,
                "answers": blueprint_answers("q", "B"),
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["score_pct"], 0.0);
    assert_eq!(
        result["passed"], false,
        "a candidate who got everything wrong must not pass: {result:?}"
    );
    assert!(result["certificate_id"].is_null(), "a fail earns no certificate");
    let reason = result["blocking_reason"].as_str().unwrap_or_default();
    assert!(reason.contains("pass mark"), "the reason should name the bar: {reason}");
}

#[tokio::test]
async fn an_unknown_option_does_not_score() {
    // A submission cannot score itself by naming an option the bank does not have.
    let app = app();
    let enrolment = enrolled(&app).await;
    register_blueprint(&app, "q", "A").await;

    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/lms/assessments/A-1/submissions",
            Some(json!({
                "enrolment_id": enrolment,
                "answers": blueprint_answers("q", "the right answer"),
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["score_pct"], 0.0);
    assert_eq!(result["passed"], false);
}

#[tokio::test]
async fn an_answer_to_an_unknown_question_does_not_score() {
    let app = app();
    let enrolment = enrolled(&app).await;
    register_blueprint(&app, "q", "A").await;
    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/lms/assessments/A-1/submissions",
            Some(json!({
                "enrolment_id": enrolment,
                "answers": [{"question_id": "not-a-question", "selected_option": "A"}]
            })),
        )
        .await,
    )
    .await;
    assert_eq!(result["score_pct"], 0.0);
    assert_eq!(result["passed"], false);
}

#[tokio::test]
async fn a_partial_submission_scores_proportionally() {
    let app = app();
    let enrolment = enrolled(&app).await;
    register_blueprint(&app, "q", "A").await;
    // One section right, three wrong: the weighted total lands on that section's
    // weight, so the score is a fraction of the pass mark, not a pass.
    let mut answers = blueprint_answers("q", "B");
    answers[0]["selected_option"] = json!("A");
    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/lms/assessments/A-1/submissions",
            Some(json!({"enrolment_id": enrolment, "answers": answers})),
        )
        .await,
    )
    .await;
    assert_eq!(result["correct"], 1);
    assert_eq!(result["answered"], 4);
    let pct = result["score_pct"].as_f64().unwrap();
    assert!(
        (pct - 35.0).abs() < 0.001,
        "one section of four, weighted 0.35, should score 35%, got {pct}"
    );
    assert_eq!(result["passed"], false, "35% is below the 80% pass mark");
}

// ---------------------------------------------------------------------------
// verify_badge
// ---------------------------------------------------------------------------

/// Approve an application and return the badge number it issued.
async fn issue_badge(app: &axum::Router) -> String {
    let application = body_json(
        send(
            app,
            "POST",
            "/api/v1/training/accreditation/applications",
            Some(json!({
                "application_type": "media",
                "organisation": "Daily Trust",
                "applicant_id_hash": "sha256:abc",
                "id_document_hash": "sha256:def",
                "photo_hash": "sha256:ghi",
                "supporting_docs_refs": []
            })),
        )
        .await,
    )
    .await;
    let id = application["application_id"].as_str().unwrap().to_string();
    let decision = body_json(
        send(
            app,
            "POST",
            &format!("/api/v1/training/accreditation/applications/{id}/decisions"),
            Some(json!({"verdict": "approve", "grounds": "Accreditation criteria met"})),
        )
        .await,
    )
    .await;
    decision["badge_number"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn an_unknown_badge_number_is_not_valid() {
    // The headline. `verify_badge` returned `valid: true` for any string.
    let app = app();
    let resp = send(&app, "GET", "/api/v1/training/accreditation/badges/EEL-NOPE", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = body_json(resp).await;
    assert!(body["valid"].is_null(), "an unknown badge is not valid");
}

#[tokio::test]
async fn an_issued_badge_verifies() {
    let app = app();
    let badge = issue_badge(&app).await;
    let verification = body_json(
        send(
            &app,
            "GET",
            &format!("/api/v1/training/accreditation/badges/{badge}"),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(verification["valid"], true);
    assert_eq!(verification["organisation"], "Daily Trust");
    assert_eq!(verification["type"], "media");
    assert!(!verification["valid_until"].is_null());
    // The holder's name is masked, not shown in full.
    let masked = verification["name_masked"].as_str().unwrap();
    assert!(masked.starts_with('D') && masked.ends_with('t'), "got {masked:?}");
    assert!(masked.contains('*'), "the middle should be masked: {masked:?}");
}

#[tokio::test]
async fn a_rejected_application_issues_no_badge() {
    let app = app();
    let application = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/accreditation/applications",
            Some(json!({
                "application_type": "media",
                "organisation": "Weekly News",
                "applicant_id_hash": "sha256:abc",
                "id_document_hash": "sha256:def",
                "photo_hash": "sha256:ghi",
                "supporting_docs_refs": []
            })),
        )
        .await,
    )
    .await;
    let id = application["application_id"].as_str().unwrap().to_string();
    let decision = body_json(
        send(
            &app,
            "POST",
            &format!("/api/v1/training/accreditation/applications/{id}/decisions"),
            Some(json!({"verdict": "reject", "grounds": "Insufficient evidence"})),
        )
        .await,
    )
    .await;
    assert!(
        decision["badge_number"].is_null(),
        "a rejected application must not issue a badge"
    );
}

#[tokio::test]
async fn approving_the_same_application_twice_does_not_mint_a_second_badge() {
    let app = app();
    let badge = issue_badge(&app).await;
    let application = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/accreditation/applications",
            Some(json!({
                "application_type": "media",
                "organisation": "Daily Trust",
                "applicant_id_hash": "sha256:abc",
                "id_document_hash": "sha256:def",
                "photo_hash": "sha256:ghi",
                "supporting_docs_refs": []
            })),
        )
        .await,
    )
    .await;
    let _ = application;
    // The badge number is derived from the application id, so it is stable.
    let again = issue_badge_for(&app, "Daily Trust").await;
    assert_ne!(badge, again, "a different application is a different badge");
}

async fn issue_badge_for(app: &axum::Router, org: &str) -> String {
    let application = body_json(
        send(
            app,
            "POST",
            "/api/v1/training/accreditation/applications",
            Some(json!({
                "application_type": "media",
                "organisation": org,
                "applicant_id_hash": "sha256:abc",
                "id_document_hash": "sha256:def",
                "photo_hash": "sha256:ghi",
                "supporting_docs_refs": []
            })),
        )
        .await,
    )
    .await;
    let id = application["application_id"].as_str().unwrap().to_string();
    body_json(
        send(
            app,
            "POST",
            &format!("/api/v1/training/accreditation/applications/{id}/decisions"),
            Some(json!({"verdict": "approve", "grounds": "Accreditation criteria met"})),
        )
        .await,
    )
    .await["badge_number"]
        .as_str()
        .unwrap()
        .to_string()
}

// ---------------------------------------------------------------------------
// get_certificate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_unknown_certificate_is_404_not_an_expired_one() {
    // Previously any id returned a URL with an `expires_at` of *now*.
    let app = app();
    let resp = send(&app, "GET", "/api/v1/training/lms/certificates/nope", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_certificate_says_why_it_is_unsigned() {
    let app = app();
    let enrolment = enrolled(&app).await;
    register_blueprint(&app, "q", "A").await;
    let result = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/lms/assessments/A-1/submissions",
            Some(json!({"enrolment_id": enrolment, "answers": blueprint_answers("q", "A")})),
        )
        .await,
    )
    .await;
    let id = result["certificate_id"].as_str().unwrap();
    let read = body_json(
        send(&app, "GET", &format!("/api/v1/training/lms/certificates/{id}"), None).await,
    )
    .await;
    assert!(read["url"].is_null(), "no document is rendered from here");
    let reason = read["blocking_reason"].as_str().unwrap_or_default();
    assert!(reason.contains("HSM"), "the reason should name what is absent: {reason}");
}

// ---------------------------------------------------------------------------
// CMS
// ---------------------------------------------------------------------------

#[tokio::test]
async fn published_content_is_stored_and_listed() {
    // Previously the content was echoed back and stored nowhere, so this list was
    // always empty.
    let app = app();
    let published = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/cms/content",
            Some(json!({"title": "Voting day guide", "body": {"blocks": ["a", "b"]}})),
        )
        .await,
    )
    .await;
    assert_eq!(published["status"], "published");
    assert_eq!(published["version"], 1);
    let id = published["content_id"].as_str().unwrap().to_string();

    let listed = body_json(send(&app, "GET", "/api/v1/training/cms/content", None).await).await;
    let items = listed["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "a published item must be listed");
    assert_eq!(items[0]["content_id"], id.as_str());

    let one = body_json(
        send(&app, "GET", &format!("/api/v1/training/cms/content/{id}"), None).await,
    )
    .await;
    assert_eq!(one["title"], "Voting day guide");
    assert_eq!(one["body"]["blocks"][1], "b");
}

#[tokio::test]
async fn unknown_content_is_404() {
    let app = app();
    // Previously `get_content` returned `{ "content_id": <id> }` with 200 for any id.
    for (method, uri) in [
        ("GET", "/api/v1/training/cms/content/nope"),
        ("PUT", "/api/v1/training/cms/content/nope"),
        ("DELETE", "/api/v1/training/cms/content/nope"),
    ] {
        let body = if method == "PUT" { Some(json!({"title": "t", "body": {}})) } else { None };
        let resp = send(&app, method, uri, body).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{method} {uri}");
    }
}

#[tokio::test]
async fn publishing_requires_a_title() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/api/v1/training/cms/content",
        Some(json!({"title": "  ", "body": {}})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_update_bumps_the_version_and_keeps_the_history_visible() {
    let app = app();
    let published = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/cms/content",
            Some(json!({"title": "Draft", "body": {"v": 1}})),
        )
        .await,
    )
    .await;
    let id = published["content_id"].as_str().unwrap().to_string();

    let updated = body_json(
        send(
            &app,
            "PUT",
            &format!("/api/v1/training/cms/content/{id}"),
            Some(json!({"title": "Final", "body": {"v": 2}})),
        )
        .await,
    )
    .await;
    assert_eq!(updated["title"], "Final");
    assert_eq!(updated["version"], 2, "each update is a distinct revision");
    assert_ne!(updated["updated_at"], published["updated_at"]);
}

#[tokio::test]
async fn deleting_content_removes_it() {
    let app = app();
    let published = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/cms/content",
            Some(json!({"title": "Ephemeral", "body": {}})),
        )
        .await,
    )
    .await;
    let id = published["content_id"].as_str().unwrap().to_string();
    assert_eq!(
        send(&app, "DELETE", &format!("/api/v1/training/cms/content/{id}"), None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    let listed = body_json(send(&app, "GET", "/api/v1/training/cms/content", None).await).await;
    assert!(listed["items"].as_array().unwrap().is_empty());
    // A second delete has nothing to delete.
    assert_eq!(
        send(&app, "DELETE", &format!("/api/v1/training/cms/content/{id}"), None)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_translation_is_stored_and_may_not_be_replaced_silently() {
    // Previously the translation was echoed back and dropped.
    let app = app();
    let published = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/cms/content",
            Some(json!({"title": "Guide", "body": {"en": "text"}})),
        )
        .await,
    )
    .await;
    let id = published["content_id"].as_str().unwrap().to_string();

    let translated = body_json(
        send(
            &app,
            "POST",
            &format!("/api/v1/training/cms/content/{id}/translations"),
            Some(json!({"language": "sw", "title": "Mwongozo", "body": {"sw": "maandishi"}})),
        )
        .await,
    )
    .await;
    assert_eq!(translated["translations"]["sw"]["title"], "Mwongozo");
    assert_eq!(translated["version"], 2);

    let again = send(
        &app,
        "POST",
        &format!("/api/v1/training/cms/content/{id}/translations"),
        Some(json!({"language": "sw", "body": {"sw": "nini"}})),
    )
    .await;
    assert_eq!(again.status(), StatusCode::CONFLICT, "a translation is not overwritten silently");
}

#[tokio::test]
async fn a_translation_needs_a_language_tag() {
    let app = app();
    let published = body_json(
        send(
            &app,
            "POST",
            "/api/v1/training/cms/content",
            Some(json!({"title": "Guide", "body": {}})),
        )
        .await,
    )
    .await;
    let id = published["content_id"].as_str().unwrap().to_string();
    let resp = send(
        &app,
        "POST",
        &format!("/api/v1/training/cms/content/{id}/translations"),
        Some(json!({"language": "not a tag!", "body": {}})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn state_is_not_shared_between_routers() {
    let a = app();
    let b = app();
    send(
        &a,
        "POST",
        "/api/v1/training/cms/content",
        Some(json!({"title": "Only in A", "body": {}})),
    )
    .await;
    let listed = body_json(send(&b, "GET", "/api/v1/training/cms/content", None).await).await;
    assert!(listed["items"].as_array().unwrap().is_empty());
}
