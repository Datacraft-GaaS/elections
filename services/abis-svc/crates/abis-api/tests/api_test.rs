use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use abis_api::{app, state::AppState};

fn test_app() -> axum::Router {
    app(Arc::new(AppState::new()))
}

fn json_body(s: &str) -> Body {
    Body::from(s.to_string())
}

#[tokio::test]
async fn test_healthz() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_match_1n() {
    let app = test_app();
    let body = r#"{"probe_ref":"p1","modality":"fingerprint","caller_id":"c1","election_id":"e1"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/abis/match/1n")
                .header("content-type", "application/json")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_match_11() {
    let app = test_app();
    let body = r#"{"probe_ref":"p1","gallery_ref":"g1","modality":"iris","caller_id":"c1"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/abis/match/11")
                .header("content-type", "application/json")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_match_result() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/abis/match/req-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_adjudication() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/abis/adjudication")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_resolve_adjudication() {
    let app = test_app();
    let body = r#"{"outcome":"no_match","adjudicator_id":"adj-1","resolution_notes":"distinct"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/v1/abis/adjudication/case-001")
                .header("content-type", "application/json")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_submit_batch_dedup() {
    let app = test_app();
    let body = r#"{"election_id":"e1","mode":"full","requester_id":"r1"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/abis/batch/dedup")
                .header("content-type", "application/json")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_get_reports() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/abis/reports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
