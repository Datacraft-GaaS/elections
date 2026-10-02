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

//! Integration tests for the online competency exam engine (§3.2):
//! lifecycle, idempotent answers, deadline + attempt enforcement, paper
//! batch import and void semantics.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use training_api::exam::{ExamBlueprint, RoleLevel, SectionSpec};
use training_api::handlers::exam::{router, ExamApiState};

async fn call(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let req = if let Some(b) = body {
        builder = builder.header("content-type", "application/json");
        builder.body(Body::from(serde_json::to_vec(&b).unwrap())).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, body)
}

fn short_blueprint() -> ExamBlueprint {
    ExamBlueprint {
        blueprint_id: "bp-test-v1".into(),
        role_code: RoleLevel::Po,
        sections: vec![SectionSpec {
            name: "law_procedure".into(),
            question_count: 2,
            weight: 1.0,
        }],
        pass_mark_pct: 50.0,
        duration_secs: 3600,
        attempts_allowed: 2,
        active_from: None,
        active_to: None,
    }
}

#[tokio::test]
async fn full_lifecycle_pass_and_get_result() {
    let app = app_with_bank_async().await;
    // Issue session.
    let (status, issued) = call(
        &app,
        "POST",
        "/v1/exams/sessions",
        Some(json!({ "application_id": "app-1", "role_code": "PO", "seed_hex": "beef" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{issued}");
    assert_eq!(issued["session"]["status"], "in_progress");
    assert_eq!(issued["session"]["attempt"], 1);
    let questions = issued["questions"].as_array().unwrap();
    assert_eq!(questions.len(), 2, "blueprint asks for 2 per section");
    for q in questions {
        assert!(q.get("options").is_some(), "options must be returned");
        assert!(q.get("correct_indexes").is_none(), "correct answers must NOT leak");
    }
    let session_id = issued["session"]["session_id"].as_str().unwrap().to_string();

    // Idempotent double PUT of the same answers is fine.
    let answers = json!({ "answers": { "q00": [0], "q01": [0], "q02": [0], "q03": [0] } });
    let (status, _) = call(&app, "PUT", &format!("/v1/exams/sessions/{session_id}/answers"), Some(answers.clone())).await;
    assert_eq!(status, StatusCode::OK);
    // Before submit a different answer REPLACES (still OK).
    let (status, _) = call(&app, "PUT", &format!("/v1/exams/sessions/{session_id}/answers"), Some(answers)).await;
    assert_eq!(status, StatusCode::OK);

    // Submit -> scored result.
    let (status, result) = call(&app, "POST", &format!("/v1/exams/sessions/{session_id}/submit"), None).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["total_pct"], 100.0);
    assert!(result["signed_receipt_hash_hex"].as_str().is_some());

    // GET result by session id.
    let (status, fetched) = call(&app, "GET", &format!("/v1/exams/results/{session_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["session_id"], session_id.as_str());

    // Post-submit PUT with a DIFFERENT answer => 409.
    let (status, err) = call(
        &app,
        "PUT",
        &format!("/v1/exams/sessions/{session_id}/answers"),
        Some(json!({ "answers": { "q00": [2] } })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{err}");
    // Post-submit PUT with the SAME answer stays idempotent.
    let (status, _) = call(
        &app,
        "PUT",
        &format!("/v1/exams/sessions/{session_id}/answers"),
        Some(json!({ "answers": { "q01": [0] } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

async fn app_with_bank_async() -> axum::Router {
    let state = ExamApiState::new();
    let bp = short_blueprint();
    {
        let mut store = state.store.write().await;
        store.blueprints.clear(); // drop defaults; only the test blueprint applies
        store.blueprints.insert(bp.blueprint_id.clone(), bp.clone());
        for i in 0..4 {
            let q: training_api::exam::Question = serde_json::from_value(json!({
                "question_id": format!("q{i:02}"),
                "role_level": "PO",
                "section": "law_procedure",
                "difficulty": 2,
                "language": "en",
                "body": format!("question {i}"),
                "options": ["a", "b", "c"],
                "correct_indexes": [0],
                "version": 1
            }))
            .unwrap();
            store.questions.insert(q.question_id.clone(), q);
        }
    }
    router(state)
}

#[tokio::test]
async fn deadline_elapsed_returns_422() {
    let state = ExamApiState::new();
    let mut bp = short_blueprint();
    bp.duration_secs = 0; // expires immediately
    {
        let mut store = state.store.write().await;
        store.blueprints.clear(); // drop defaults; only the test blueprint applies
        store.blueprints.insert(bp.blueprint_id.clone(), bp.clone());
        for i in 0..2 {
            let q: training_api::exam::Question = serde_json::from_value(json!({
                "question_id": format!("q{i:02}"),
                "role_level": "PO",
                "section": "law_procedure",
                "difficulty": 2,
                "language": "en",
                "body": "x",
                "options": ["a", "b"],
                "correct_indexes": [0],
                "version": 1
            }))
            .unwrap();
            store.questions.insert(q.question_id.clone(), q);
        }
    }
    let app = router(state);
    let (_, issued) = call(
        &app,
        "POST",
        "/v1/exams/sessions",
        Some(json!({ "application_id": "app-late", "role_code": "PO" })),
    )
    .await;
    let session_id = issued["session"]["session_id"].as_str().unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let (status, body) = call(&app, "POST", &format!("/v1/exams/sessions/{session_id}/submit"), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "deadline_elapsed");

    // The late submit must not have produced a score.
    let (status, _) = call(&app, "GET", &format!("/v1/exams/results/{session_id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn attempts_exhausted_returns_409_and_void_consumes_attempt() {
    let app = app_with_bank_async().await; // attempts_allowed = 2
    let issue = |aid: &str| {
        let app = app.clone();
        let aid = aid.to_string();
        async move {
            call(
                &app,
                "POST",
                "/v1/exams/sessions",
                Some(json!({ "application_id": aid, "role_code": "PO" })),
            )
            .await
        }
    };

    // Attempt 1: void it — voiding CONSUMES the attempt.
    let (_, first) = issue("app-x").await;
    let sid1 = first["session"]["session_id"].as_str().unwrap().to_string();
    let (status, _) = call(&app, "POST", &format!("/v1/exams/sessions/{sid1}/void"), Some(json!({ "reason": "board review" }))).await;
    assert_eq!(status, StatusCode::OK);

    // Attempt 2 still available despite void.
    let (status, second) = issue("app-x").await;
    assert_eq!(status, StatusCode::CREATED, "{second}");
    assert_eq!(second["session"]["attempt"], 2, "void consumes an attempt");
    let sid2 = second["session"]["session_id"].as_str().unwrap();

    // Third attempt exhausted => 409.
    let (status, err) = issue("app-x").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(err["error"], "attempts_exhausted");

    // Independent application unaffected.
    let (status, _) = issue("app-y").await;
    assert_eq!(status, StatusCode::CREATED);
    let _ = sid2;
}

#[tokio::test]
async fn paper_batch_import_creates_scored_results_marked_paper() {
    let app = router(ExamApiState::new()); // default blueprints include PO (80% pass)
    let (status, resp) = call(
        &app,
        "POST",
        "/v1/exams/paper-batch",
        Some(json!({
            "blueprint_id": "bp-po-v1",
            "entries": [
                { "application_id": "paper-1", "total_pct": 85.0 },
                { "application_id": "paper-2", "total_pct": 60.0 }
            ]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{resp}");
    let results = resp["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["channel"], "paper");
    assert!(results[0]["passed"].as_bool().unwrap());
    assert!(!results[1]["passed"].as_bool().unwrap(), "60% < PO pass mark 80%");
    assert!(results[0]["signed_receipt_hash_hex"].as_str().is_some());

    // Fetchable via results endpoint.
    let sid = results[1]["session_id"].as_str().unwrap();
    let (status, fetched) = call(&app, "GET", &format!("/v1/exams/results/{sid}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["channel"], "paper");
}

#[tokio::test]
async fn blueprints_endpoint_filters_by_role() {
    let app = router(ExamApiState::new());
    let (status, all) = call(&app, "GET", "/v1/exams/blueprints", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(all["blueprints"].as_array().unwrap().len(), 3);

    let (status, clerks) = call(&app, "GET", "/v1/exams/blueprints?role_code=CLERK", None).await;
    assert_eq!(status, StatusCode::OK);
    let list = clerks["blueprints"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["pass_mark_pct"], 70.0);
}
