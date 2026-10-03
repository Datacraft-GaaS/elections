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
use printing::http::{router, AppState};
use tower::ServiceExt;

#[tokio::test]
async fn router_smoke() {
    let state = AppState::new_shared();
    let app = router(state);
    drop(app);
}

#[tokio::test]
async fn contract_create_and_get_print_order() {
    let state = AppState::new_shared();
    let app = router(state);

    // Create
    let req_body = serde_json::json!({
        "election_id": uuid::Uuid::now_v7().to_string(),
        "position_code": "president",
        "constituency_code": "001",
        "ballot_quantity": 1000,
        "proof_id": uuid::Uuid::now_v7().to_string(),
        "security_features": ["uv_ink"]
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/printing/print-orders")
        .header("content-type", "application/json")
        .body(Body::from(req_body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(created["order_id"].is_string());
    let order_id = created["order_id"].as_str().unwrap().to_string();

    // Get
    let req = Request::builder()
        .uri(format!("/api/v1/printing/print-orders/{order_id}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // QC
    let qc_body = serde_json::json!({
        "passed": true,
        "defect_count": 0,
        "sample_size": 100,
        "qc_officer_hash": "abc",
        "decided_at": chrono::Utc::now().to_rfc3339()
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/printing/print-orders/{order_id}/qc"))
        .header("content-type", "application/json")
        .body(Body::from(qc_body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}
