// portal-api integration tests: rate-limit, cache-poisoning, Merkle round-trip.
// (M11-SEC-, M11-INT-, M11-UNIT-)

use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

/// Cache-poisoning defence (AC-M11-05): snapshot responses must carry signing
/// headers so the edge can key caches on merkle_root and snapshot_id.
/// A snapshot without these headers must be rejected by clients.
#[tokio::test]
async fn cache_poisoning_defence_signing_headers_present() {
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/latest")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // This asserted the presence of X-Eel-Merkle-Root and X-Eel-Snapshot-Id, which
    // is what let a 64-zero root reach the edge: the handler answered 200 with
    // Uuid::nil() and `stale: false`, and those two headers carried the zero to the
    // cache the defence exists to protect. There is no snapshot to respond with, so
    // the defence is a refusal -- and no root is published at all.
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let hdrs = resp.headers();
    assert!(
        !hdrs.contains_key("x-eel-merkle-root"),
        "no snapshot means no cache key; an all-zero root would map every snapshot \
         onto one cache entry"
    );
    // The MIME-sniffing defence is global now, not snapshot-only.
    assert_eq!(
        hdrs.get("x-content-type-options").map(|v| v.as_bytes()),
        Some(b"nosniff" as &[u8])
    );
}

/// Rate-limit burst: 20 rapid requests must not produce any 5xx.
/// Edge WAF (ADR-0010) is the primary gate; API layer must stay stable.
#[tokio::test]
async fn burst_does_not_produce_5xx() {
    for _ in 0..20 {
        let app = portal_api::app();
        let req = Request::builder()
            .uri("/api/v1/portal/snapshots/00000000-0000-0000-0000-000000000000/verify")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // 501 is deliberate and not a 5xx in the sense this test means: the
        // endpoint refuses rather than serving a fabricated proof.
        assert!(
            resp.status().as_u16() == 501,
            "unexpected status: {}",
            resp.status()
        );
    }
}

/// Merkle proof round-trip: verify endpoint returns valid JSON proof shape.
#[tokio::test]
async fn merkle_proof_json_shape() {
    use axum::body::to_bytes;
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/11111111-1111-1111-1111-111111111111/verify")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), 65_536).await.unwrap();
    let proof: serde_json::Value = serde_json::from_slice(&body).expect("valid JSON");
    // Previously 200 with a proof assembled from placeholders: a 64-zero
    // leaf_hash, a 64-zero root, no siblings, tree_size 1 and an empty sth_sig.
    assert_eq!(status.as_u16(), 501);
    for field in ["leaf_hash", "siblings", "root", "verified", "tree_size"] {
        assert!(
            proof.get(field).is_none(),
            "a refusal must not carry {field}"
        );
    }
}

/// Mismatched UUID in path returns 400, not 500.
#[tokio::test]
async fn bad_uuid_path_is_400_not_500() {
    let app = portal_api::app();
    let req = Request::builder()
        .uri("/api/v1/portal/snapshots/this-is-not-a-uuid/verify")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 400);
}
