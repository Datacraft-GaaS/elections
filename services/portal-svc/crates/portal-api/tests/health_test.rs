use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn snapshots_latest_returns_ok() {
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/latest")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("x-eel-merkle-root"));
    assert!(resp.headers().contains_key("x-eel-snapshot-id"));
}

#[tokio::test]
async fn snapshot_bad_uuid_returns_400() {
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/not-a-uuid")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn snapshots_bulk_does_not_redirect_to_a_dead_host() {
    // This asserted TEMPORARY_REDIRECT, which is what pinned the fabrication: the
    // redirect target was a hardcoded path on minio.eel.internal, which does not
    // resolve. Object storage is not in this workspace, so there is no presigned
    // URL to hand out and the endpoint says so.
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/bulk.tar.gz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    assert!(resp.headers().get("location").is_none());
}

/// Cache-poisoning defence (AC-M11-05): a snapshot response must carry
/// X-Eel-Merkle-Root and X-Eel-Snapshot-Id so edge caches key on them.
/// A response without those headers must be rejected by clients.
#[tokio::test]
async fn snapshot_response_carries_signing_headers() {
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/latest")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let hdrs = resp.headers();
    assert!(
        hdrs.contains_key("x-eel-merkle-root"),
        "missing X-Eel-Merkle-Root header"
    );
    assert!(
        hdrs.contains_key("x-eel-snapshot-id"),
        "missing X-Eel-Snapshot-Id header"
    );
    // X-Content-Type-Options must be present to prevent MIME sniffing.
    assert_eq!(
        hdrs.get("x-content-type-options").map(|v| v.as_bytes()),
        Some(b"nosniff" as &[u8])
    );
}

/// Merkle proof round-trip: verify endpoint returns a JSON body with
/// expected fields (leaf_hash, siblings, root, verified).
#[tokio::test]
async fn merkle_proof_round_trip() {
    use axum::body::to_bytes;
    let app = portal_api::app();
    let snap_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/api/v1/portal/snapshots/{snap_id}/verify"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 65_536).await.unwrap();
    let proof: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(proof.get("leaf_hash").is_some());
    assert!(proof.get("siblings").is_some());
    assert!(proof.get("root").is_some());
    assert!(proof.get("verified").is_some());
}
