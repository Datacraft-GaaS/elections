use std::sync::Arc;

#[tokio::test]
async fn health_returns_ok() {
    let app = rts_api::router::build_router(Arc::new(rts_api::store::AppState::default()));
    let resp = app.oneshot(http_request()).await.unwrap();
    assert_eq!(resp.status(), 200);
}

fn http_request() -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .uri("/health")
        .body(axum::body::Body::empty())
        .unwrap()
}

use tower::ServiceExt;
