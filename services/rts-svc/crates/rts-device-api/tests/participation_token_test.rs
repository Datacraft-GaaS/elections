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

//! A participation token is not issued against a kit that does not exist.
//!
//! The handler returned `kit_serial: "STUB-KIT"` and a `token_sha256` computed as
//! `format!("{:064x}", Uuid::new_v4().as_u128())` — a UUID's 128 bits zero-padded
//! to look like a SHA-256 digest, which is 256 bits of the wrong thing. It read as
//! a hash and was not one, so nothing could ever have verified it.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn a_participation_token_is_not_issued_without_a_kit() {
    let app = rts_device_api::router::build_router();
    let req = Request::builder()
        .uri("/api/v1/rts/device/participation-tokens/NRB-001-PS-042")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    // The old response's two lies are both gone.
    assert!(body.get("kit_serial").is_none(), "no stub kit serial");
    assert!(
        body.get("token_sha256").is_none(),
        "no digest that is not a digest"
    );
    assert!(body["detail"].as_str().unwrap().contains("kit registry"));
}

#[tokio::test]
async fn no_station_is_ever_issued_a_token() {
    // There must be no code path that answers 200 here. A token for an arbitrary
    // station code is exactly the failure: any input produced a plausible-looking
    // digest.
    for station in ["NRB-001-PS-042", "ANY-STATION", "047-001-001"] {
        let app = rts_device_api::router::build_router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/rts/device/participation-tokens/{station}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_IMPLEMENTED,
            "station {station:?} must not be issued a token"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(body.get("token_sha256").is_none(), "station {station:?}");
        assert!(body.get("kit_serial").is_none(), "station {station:?}");
    }
}
