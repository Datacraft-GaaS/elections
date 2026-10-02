//! A bulk snapshot download is not advertised when object storage is absent.
//!
//! The handler redirected to `https://minio.eel.internal/portal-snapshots/latest.tar.gz`
//! — a hardcoded path on a host that does not resolve. A 302 is a *successful*
//! answer: a client follows it and gets a DNS failure, having been told the
//! snapshot was ready to download. The old comment said "Stub returns 302", which
//! was the honest description.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn the_bulk_snapshot_does_not_redirect_to_a_host_that_cannot_resolve() {
    let app = portal_api::app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/portal/snapshots/bulk.tar.gz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(
        resp.status(),
        StatusCode::FOUND,
        "a 302 sends the client to minio.eel.internal, which does not resolve"
    );
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    // The Location header, if any, must not name the dead host.
    assert!(resp.headers().get("location").is_none());
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], 501);
    assert!(body["detail"].as_str().unwrap().contains("MinIO"));
}
