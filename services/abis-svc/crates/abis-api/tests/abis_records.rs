//! Round-trip tests for the ABIS API's fabricated biometric records.
//!
//! - `get_enrolment_job` returned `subject_id: "subject-001"`,
//!   `status: "processing"` and `modalities_completed: ["fingerprint"]` for
//!   **any** id — a fabricated subject identity and capture state.
//! - `get_quality_report` returned `fingerprint_nfiq2: 60.0`, `face_quality: 0.92`,
//!   `iris_quality: 0.88` and **`overall_pass: true`** for any id. A passing quality
//!   report is the record on which an enrolment proceeds.
//! - `withdraw_enrolment_job` returned `status: "withdrawn"` for any id.
//! - `get_fmr_fnmr` returned `fmr: 0.001, fnmr: 0.005` as constants — the figures
//!   that decide whether a biometric system is fit for electoral use.
//! - `get_dedup_summary` returned zeros, `get_fairness` empty groups, and
//!   `get_enrolment_throughput` zeroed percentiles: each reads as "nothing was
//!   found" or "nothing abnormal" rather than "nothing was measured".
//! - `list_vendors` returned a `Mock Vendor` reported `healthy`.
//! - `readyz` answered `ready: true` with every check `"ok"` without touching a
//!   database, a vendor or a matcher.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use abis_api::{app as build_app, state::AppState, store::AbisStore};
use std::sync::Arc;

fn app() -> axum::Router {
    let state = AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://127.0.0.1:1/none")
            .expect("connect_lazy cannot fail"),
        store: AbisStore::new(),
    };
    build_app(Arc::new(state))
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

/// Create a job and return its id.
async fn create_job(app: &axum::Router, subject: &str, modalities: &[&str]) -> String {
    let resp = send(
        app,
        "POST",
        "/v1/enrolment/jobs",
        Some(json!({
            "subject_id": subject,
            "modalities": modalities,
            "operator_id": "operator-1"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED, "a job must be accepted");
    body_json(resp).await["job_id"].as_str().unwrap().to_string()
}

// ---------------------------------------------------------------------------
// enrolment jobs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_unknown_job_is_not_a_processing_subject() {
    // The old handler answered 200 with `subject_id: "subject-001"` and
    // `status: "processing"` for any id.
    let app = app();
    let resp = send(&app, "GET", "/v1/enrolment/jobs/job-nope", None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "a job id that was never issued must not read as a processing subject"
    );
}

#[tokio::test]
async fn a_created_job_reads_back_with_its_own_subject() {
    let app = app();
    let id = create_job(&app, "subject-abc", &["fingerprint", "face"]).await;
    let job = body_json(send(&app, "GET", &format!("/v1/enrolment/jobs/{id}"), None).await).await;
    assert_eq!(job["job_id"], id.as_str());
    assert_eq!(job["subject_id"], "subject-abc");
    assert_eq!(job["status"], "queued");
    assert_eq!(job["modalities_completed"], json!([]));
    assert_eq!(job["requested_modalities"], json!(["fingerprint", "face"]));
    assert!(job["created_at"].is_string());
}

#[tokio::test]
async fn a_job_needs_a_subject_and_a_modality() {
    let app = app();
    for body in [
        json!({"subject_id": "  ", "modalities": ["fingerprint"], "operator_id": "o"}),
        json!({"subject_id": "s", "modalities": [], "operator_id": "o"}),
    ] {
        let resp = send(&app, "POST", "/v1/enrolment/jobs", Some(body)).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn two_jobs_have_different_subjects() {
    let app = app();
    let a = create_job(&app, "subject-a", &["fingerprint"]).await;
    let b = create_job(&app, "subject-b", &["fingerprint"]).await;
    let ja = body_json(send(&app, "GET", &format!("/v1/enrolment/jobs/{a}"), None).await).await;
    let jb = body_json(send(&app, "GET", &format!("/v1/enrolment/jobs/{b}"), None).await).await;
    assert_eq!(ja["subject_id"], "subject-a");
    assert_eq!(jb["subject_id"], "subject-b");
    assert_ne!(a, b);
}

// ---------------------------------------------------------------------------
// quality report
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_quality_report_for_an_unknown_job_is_404() {
    // The old handler returned `overall_pass: true` for any id.
    let app = app();
    let resp = send(&app, "GET", "/v1/enrolment/jobs/job-nope/quality-report", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_quality_report_is_not_issued_before_capture_completes() {
    let app = app();
    let id = create_job(&app, "subject-abc", &["fingerprint", "face"]).await;
    let resp = send(
        &app,
        "GET",
        &format!("/v1/enrolment/jobs/{id}/quality-report"),
        None,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "a job with no captured modalities has no quality report"
    );
    let body = body_json(resp).await;
    assert_eq!(body["status"], "queued");
}

// ---------------------------------------------------------------------------
// withdrawal
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_unknown_job_cannot_be_withdrawn() {
    // The old handler returned `status: "withdrawn"` for any id.
    let app = app();
    let resp = send(&app, "PATCH", "/v1/enrolment/jobs/job-nope/withdraw", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_queued_job_can_be_withdrawn_once() {
    let app = app();
    let id = create_job(&app, "subject-abc", &["fingerprint"]).await;
    let first = send(&app, "PATCH", &format!("/v1/enrolment/jobs/{id}/withdraw"), None).await;
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(body_json(first).await["status"], "withdrawn");

    let second = send(&app, "PATCH", &format!("/v1/enrolment/jobs/{id}/withdraw"), None).await;
    assert_ne!(second.status(), StatusCode::OK, "a withdrawn job does not reopen");
}

// ---------------------------------------------------------------------------
// FMR / FNMR
// ---------------------------------------------------------------------------

#[tokio::test]
async fn fmr_and_fnmr_refuse_rather_than_report_a_plausible_pair() {
    // The old handler returned `fmr: 0.001, fnmr: 0.005` as constants. Those two
    // figures decide whether a biometric system is fit for electoral use, and a
    // reader cannot tell a plausible constant from a measured value. This is now
    // an explicit refusal carrying the reason, matching `get_reports`.
    let app = app();
    let resp = send(&app, "GET", "/v1/reports/fmr-fnmr", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("test corpus"), "{detail}");
    assert!(detail.contains("0.001"), "the reason should name what it replaced: {detail}");
}

#[tokio::test]
async fn fmr_and_fnmr_refuse_for_any_slice() {
    let app = app();
    let resp = send(&app, "GET", "/v1/reports/fmr-fnmr?slice=by-county", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
}

// ---------------------------------------------------------------------------
// dedup, fairness, throughput
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_dedup_summary_reports_no_comparisons_rather_than_no_duplicates() {
    // The old handler returned zeros, which reads as "no duplicates were found".
    let app = app();
    create_job(&app, "subject-abc", &["fingerprint"]).await;
    let summary = body_json(send(&app, "GET", "/v1/reports/dedup-summary", None).await).await;
    assert!(summary["total_comparisons"].is_null());
    assert!(summary["duplicate_rate_pct"].is_null());
    assert_eq!(summary["measured"], false);
    // A real count of this service's own records.
    assert_eq!(summary["enrolment_jobs_captured"], 0);
}

#[tokio::test]
async fn a_fairness_report_refuses_rather_than_reporting_no_disparity() {
    // The old handler returned empty groups, which reads as "no disparity".
    let app = app();
    let resp = send(&app, "GET", "/v1/reports/fairness", None).await;
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(detail.contains("demographic"), "{detail}");
}

#[tokio::test]
async fn throughput_latency_is_null_rather_than_zero() {
    // The old handler returned `p50_ms: 0, p99_ms: 0`, which reads as a measured
    // latency distribution.
    let app = app();
    create_job(&app, "subject-abc", &["fingerprint"]).await;
    let report = body_json(send(&app, "GET", "/v1/reports/enrolment-throughput", None).await).await;
    assert!(report["p50_ms"].is_null());
    assert!(report["p99_ms"].is_null());
    assert_eq!(report["measured"], false);
    assert_eq!(report["jobs_accepted_in_window"], 1, "a real count");
    assert_eq!(report["enrolments_per_hour"], 0, "nothing captured yet");
}

// ---------------------------------------------------------------------------
// vendors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn the_vendor_list_does_not_contain_a_healthy_mock() {
    // The old handler returned a single `Mock Vendor` reported `healthy` with
    // fingerprint, face and iris capabilities.
    let app = app();
    let vendors = body_json(send(&app, "GET", "/v1/admin/vendors", None).await).await;
    assert!(
        vendors.as_array().unwrap().is_empty(),
        "an empty list is honest; a fabricated vendor reported healthy is not: {vendors:?}"
    );
}

#[tokio::test]
async fn a_vendor_appears_once_a_kit_from_it_enrols() {
    let app = app();
    send(
        &app,
        "POST",
        "/v1/enrolment/jobs",
        Some(json!({
            "subject_id": "s",
            "modalities": ["fingerprint"],
            "operator_id": "o",
            "demographics": {"vendor_id": "vendor-x"}
        })),
    )
    .await;
    let vendors = body_json(send(&app, "GET", "/v1/admin/vendors", None).await).await;
    let vendors = vendors.as_array().unwrap();
    assert_eq!(vendors.len(), 1);
    assert_eq!(vendors[0]["id"], "vendor-x");
    assert_eq!(vendors[0]["enrolled_count"], 1);
    assert_eq!(vendors[0]["status"], "registered");
    assert!(!vendors[0]["last_seen_at"].is_null());
}

// ---------------------------------------------------------------------------
// readiness
// ---------------------------------------------------------------------------

#[tokio::test]
async fn readiness_is_not_assured_without_a_database() {
    // The old handler answered `ready: true` with every check "ok" without
    // touching a database, a vendor or a matcher.
    let app = app();
    let resp = send(&app, "GET", "/readyz", None).await;
    assert_eq!(
        resp.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "a probe that always passes keeps a broken instance in the load-balancer pool"
    );
    let body = body_json(resp).await;
    assert_eq!(body["ready"], false);
    let checks = &body["checks"];
    assert_ne!(
        checks["database"], "ok",
        "the database is unreachable, and the probe says so: {checks:?}"
    );
    assert!(checks["database"].is_string());
}

#[tokio::test]
async fn readiness_does_not_claim_the_matcher_is_ok() {
    let app = app();
    let resp = send(&app, "GET", "/readyz", None).await;
    let checks = body_json(resp).await["checks"].clone();
    assert_eq!(
        checks["matcher"], "not-performed",
        "the matcher is a separate service; reporting it ok is the same fabrication: {checks:?}"
    );
    assert_ne!(checks["vendor"], "ok");
}

#[tokio::test]
async fn the_liveness_probe_still_answers() {
    // `/healthz` is liveness, not readiness: the process being up is a real thing
    // it can report, so it is not gated on the database.
    let app = app();
    let resp = send(&app, "GET", "/healthz", None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["ok"], true);
}

#[tokio::test]
async fn routers_do_not_share_state() {
    let a = app();
    let b = app();
    let id = create_job(&a, "subject-a", &["fingerprint"]).await;
    let resp = send(&b, "GET", &format!("/v1/enrolment/jobs/{id}"), None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// modality capture: the only path to a completed job
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_captured_job_still_has_no_quality_pass_verdict() {
    // Reaching this point requires the capture to complete, and even then there is
    // no `overall_pass: true`. The three figures come from the kit's sensor and
    // the matcher, neither of which is reachable from this process.
    let app = app();
    let id = create_job(&app, "subject-abc", &["fingerprint", "face"]).await;
    for modality in ["fingerprint", "face"] {
        let resp = send(
            &app,
            "POST",
            &format!("/v1/enrolment/jobs/{id}/modalities"),
            Some(json!({"modality": modality, "subject_id_hash": "sha256:abc"})),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    let job = body_json(send(&app, "GET", &format!("/v1/enrolment/jobs/{id}"), None).await).await;
    assert_eq!(job["status"], "captured");
    assert_eq!(job["modalities_completed"], json!(["fingerprint", "face"]));

    let quality = body_json(
        send(
            &app,
            "GET",
            &format!("/v1/enrolment/jobs/{id}/quality-report"),
            None,
        )
        .await,
    )
    .await;
    assert!(quality["overall_pass"].is_null(), "{quality:?}");
    assert!(quality["fingerprint_nfiq2"].is_null());
    assert!(quality["face_quality"].is_null());
    assert!(quality["iris_quality"].is_null());
    assert_eq!(quality["measured"], Value::Null);
    let reason = quality["blocking_reason"].as_str().unwrap_or_default();
    assert!(reason.contains("no pass verdict"), "{reason}");
}

#[tokio::test]
async fn a_captured_job_cannot_be_withdrawn() {
    // The templates are already in the system; withdrawing would imply they are not.
    let app = app();
    let id = create_job(&app, "subject-abc", &["fingerprint"]).await;
    send(
        &app,
        "POST",
        &format!("/v1/enrolment/jobs/{id}/modalities"),
        Some(json!({"modality": "fingerprint", "subject_id_hash": "sha256:abc"})),
    )
    .await;
    let resp = send(&app, "PATCH", &format!("/v1/enrolment/jobs/{id}/withdraw"), None).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let job = body_json(send(&app, "GET", &format!("/v1/enrolment/jobs/{id}"), None).await).await;
    assert_eq!(job["status"], "captured");
}

#[tokio::test]
async fn an_unrequested_modality_is_refused() {
    // Recording a modality the kit was never asked for would inflate a job's
    // capture record.
    let app = app();
    let id = create_job(&app, "subject-abc", &["fingerprint"]).await;
    let resp = send(
        &app,
        "POST",
        &format!("/v1/enrolment/jobs/{id}/modalities"),
        Some(json!({"modality": "iris", "subject_id_hash": "sha256:abc"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let job = body_json(send(&app, "GET", &format!("/v1/enrolment/jobs/{id}"), None).await).await;
    assert_eq!(job["modalities_completed"], json!([]));
}

#[tokio::test]
async fn a_modality_for_an_unknown_job_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/enrolment/jobs/job-nope/modalities",
        Some(json!({"modality": "fingerprint", "subject_id_hash": "sha256:abc"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
