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

//! Contract tests for portal-api (public results/turnout/proof endpoints).
//!
//! Pins status codes + response shapes from `portal/public.yaml`. Snapshot
//! handlers currently serve well-typed placeholders (zeroed Merkle roots,
//! staged 302s) pending Valkey/MinIO wiring; these tests lock the shapes so
//! that wiring cannot silently change the contract.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use portal_api::app;
use serde_json::Value;
use tower::ServiceExt;

async fn get(uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri(uri)
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn health_shape() {
    let (status, body) = get("/api/v1/portal/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert_eq!(body["service"], "portal-api");
}

#[tokio::test]
async fn latest_snapshot_is_not_reported_as_a_zero_root() {
    // This asserted 200 with a merkle_root and snapshot_id present, which is what
    // pinned the fabrication: the handler returned Uuid::nil(), a 64-zero
    // merkle_root, an empty signature and `stale: false`. It said the newest
    // snapshot does not exist and is not stale, and it put the zero root in an
    // X-Eel-Merkle-Root header that edge caches are meant to key on (AC-M11-05).
    let (status, body) = get("/api/v1/portal/snapshots/latest").await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(body["status"], 501);
    assert!(body["detail"].as_str().unwrap().contains("Uuid::nil()"));
}

#[tokio::test]
async fn bulk_does_not_redirect_when_there_is_no_object_store() {
    // This asserted TEMPORARY_REDIRECT, which is what pinned the fabrication: the
    // handler redirected to `https://minio.eel.internal/portal-snapshots/latest.tar.gz`,
    // a hardcoded path on a host that does not resolve. A 302 tells a client the
    // download is ready and then fails at the host name.
    let (status, body) = get("/api/v1/portal/snapshots/bulk.tar.gz").await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(body["status"], 501);
}

#[tokio::test]
async fn snapshot_by_id_rejects_non_uuid() {
    let (status, _) = get("/api/v1/portal/snapshots/not-a-uuid").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn snapshot_by_id_is_not_echoed_back_as_if_it_existed() {
    // This asserted 200 with the caller's own id echoed back, alongside a zero
    // root. It is an addressed-looking response for a snapshot that does not exist,
    // and there is no store here to look the id up against.
    let (status, body) =
        get("/api/v1/portal/snapshots/123e4567-e89b-12d3-a456-426614174000").await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(body.get("merkle_root").is_none());
}

#[tokio::test]
async fn national_results_shape() {
    let (status, body) = get("/api/v1/portal/results").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("results").is_some());
}

#[tokio::test]
async fn live_turnout_shape() {
    let (status, body) = get("/api/v1/portal/turnout").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("turnout_pct").is_some());
}
