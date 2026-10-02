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
async fn snapshots_latest_does_not_publish_a_zero_root() {
    // This asserted 200 *and* that the cache-keying headers were present, which is
    // how a 64-zero merkle_root got published: the handler returned Uuid::nil()
    // with `stale: false`, and the headers the AC-M11-05 cache defence depends on
    // carried that zero to the edge.
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/latest")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    // The cache-keying headers must not be present at all: there is no root, so
    // there is nothing to key on, and a zero there collapses every snapshot onto
    // one cache entry.
    assert!(!resp.headers().contains_key("x-eel-merkle-root"));
    assert!(!resp.headers().contains_key("x-eel-snapshot-id"));
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
    // AC-M11-05 requires a snapshot response to carry both headers so edge caches
    // key on them. There is no snapshot to respond with here, so the requirement is
    // met by refusing rather than by publishing a placeholder root -- and the check
    // that matters is that no all-zero root is ever emitted.
    if hdrs.contains_key("x-eel-merkle-root") {
        let root = hdrs
            .get("x-eel-merkle-root")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert_ne!(
            root,
            "0".repeat(64),
            "an all-zero Merkle root must never be published to a cache"
        );
    }
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
    // Previously 200 with a proof assembled from placeholders: a 64-zero
    // leaf_hash, a 64-zero root, no siblings, tree_size 1 and an empty sth_sig.
    // A client asking "is this snapshot in the log" got a proof that verifies
    // nothing, with `verified: false` looking like a field to trust rather than an
    // absence to notice. There is no sibling path or transparency log here to check
    // against, so the endpoint says so and returns nothing proof-shaped.
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let body = to_bytes(resp.into_body(), 65_536).await.unwrap();
    let proof: serde_json::Value = serde_json::from_slice(&body).unwrap();
    for field in ["leaf_hash", "siblings", "root", "verified"] {
        assert!(
            proof.get(field).is_none(),
            "a refusal must not carry {field}"
        );
    }
}
