use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[cfg(test)] // trace: ["FR-M37-001","FR-M42-016","FR-M60-011"]

#[tokio::test]
async fn health_returns_200() {
    let app = training_api::router::build_router();
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_content_returns_201() {
    let app = training_api::router::build_router();
    let body = serde_json::json!({
        "title": "How to Vote",
        "language": "sw",
        "format": "video",
        "topic_category": "voter-education",
        "target_audience": "first-time-voters",
        "election_cycle_tag": "2027-general"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/v1/training/cms/content")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn report_medical_incident_returns_201() {
    let app = training_api::router::build_router();
    let body = serde_json::json!({
        "station_id": "STN-001",
        "incident_type": "cardiac",
        "severity": "high",
        "persons_affected": 1
    });
    let req = Request::builder()
        .method("POST")
        .uri("/v1/training/medical/incidents")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn verify_badge_returns_no_pii() {
    let app = training_api::router::build_router();
    let req = Request::builder()
        .uri("/v1/training/badges/BADGE-001/verify")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body.get("applicant_name").is_none(), "response must not contain applicant_name PII");
    assert!(body.get("id_number").is_none(), "response must not contain id_number PII");
}

#[tokio::test]
async fn get_crl_returns_revoked_serials_array() {
    let app = training_api::router::build_router();
    let req = Request::builder()
        .uri("/v1/training/crl")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body.get("revoked_serials").is_some(), "response must contain revoked_serials array");
    assert!(body["revoked_serials"].is_array(), "revoked_serials must be an array");
}
