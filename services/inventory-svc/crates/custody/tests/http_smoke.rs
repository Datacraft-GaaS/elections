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

use axum::{body::Body, http::Request};
use custody::{router, state::AppState};
use tower::ServiceExt;

#[tokio::test]
async fn router_smoke() {
    let state = AppState::new();
    let app = router::router(state);
    drop(app);
}

#[tokio::test]
async fn contract_register_and_get_asset() {
    let state = AppState::new();
    let app = router::router(state);

    // Create
    let req_body = serde_json::json!({
        "asset_type": "ballot_box",
        "serial_number": "BB-2026-00142",
        "current_custodian_id": "off-8f3a2d1e",
        "current_location": "CMS-NRB-CENTRAL",
        "condition": "new"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/custody/assets")
        .header("content-type", "application/json")
        .body(Body::from(req_body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(created["asset_id"].is_string());
    let asset_id = created["asset_id"].as_str().unwrap().to_string();

    // Get
    let req = Request::builder()
        .uri(format!("/api/v1/custody/assets/{asset_id}/history"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}
