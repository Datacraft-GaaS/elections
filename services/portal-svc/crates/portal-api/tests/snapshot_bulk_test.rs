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
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], 501);
    assert!(body["detail"].as_str().unwrap().contains("MinIO"));
}

#[tokio::test]
async fn a_snapshot_inclusion_proof_is_not_handed_out_against_a_zero_root() {
    // The handler returned 200 with a proof built entirely from placeholders: a
    // 64-zero leaf_hash, a 64-zero root, no siblings, tree_size 1 and an empty
    // sth_sig. A client asking "is this snapshot in the log" got a proof that
    // verifies nothing, with `verified: false` sitting there looking like a field
    // to trust rather than an absence to notice.
    let app = portal_api::app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/portal/snapshots/123e4567-e89b-12d3-a456-426614174000/verify")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    // Nothing that a client could mistake for a proof.
    for field in ["leaf_hash", "root", "siblings", "sth_sig", "verified"] {
        assert!(body.get(field).is_none(), "no {field} in the response");
    }
}
