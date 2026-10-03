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

//! A config bundle is not signed with the literal string "stub-sig".
//!
//! `get_config_default` answered 200 with `station_code: "unknown"`,
//! `config_version: 1` and `manifest_sig: "stub-sig"`. A kit that verifies a config
//! bundle has no way to tell that from a real signature except by knowing the
//! manifest, which it does not -- the signature is what it trusts.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn app() -> axum::Router {
    edge_api::routes::config::router()
}

#[tokio::test]
async fn the_default_bundle_is_refused_rather_than_stub_signed() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/api/v1/biometric-edge/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&bytes).to_string();
    assert!(!text.contains("stub-sig"), "{text}");
    // And nothing that a kit would parse as a bundle.
    let body: serde_json::Value = serde_json::from_slice(text.as_bytes()).unwrap_or_default();
    for field in ["station_code", "manifest_sig", "payload_url"] {
        assert!(body.get(field).is_none(), "no {field} in a refusal");
    }
}

#[tokio::test]
async fn no_response_from_this_router_ever_carries_a_stub_signature() {
    for uri in [
        "/api/v1/biometric-edge/config",
        "/api/v1/edge/register-extract/NRB-001-PS-042",
    ] {
        let resp = app()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8_lossy(&bytes).to_string();
        assert!(!text.contains("stub-sig"), "{uri}: {text}");
    }
}
