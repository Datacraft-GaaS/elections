use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200() {
    let app = ecc_api::router::build_router();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn situational_awareness_returns_200() {
    let app = ecc_api::router::build_router();
    let req = Request::builder()
        .uri("/ecc/situational-awareness")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_complaints_returns_200() {
    let app = ecc_api::router::build_router();
    let req = Request::builder()
        .uri("/grievance/complaints")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
