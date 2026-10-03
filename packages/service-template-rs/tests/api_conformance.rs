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

//! Conformance test for the service-template-rs package itself.
//!
//! Spins up an `ApiServer` with no app routes and verifies the shared suite passes.

use axum::Router;
use eel_api_server_rs::ApiServer;
use reqwest::Client;

#[tokio::test]
async fn conformance_health_suite_passes() {
    let server = ApiServer::builder()
        .jwks_url("http://localhost/jwks-nonexistent")
        .otel_service_name("conformance-test")
        .build()
        .expect("build");

    let router = server.into_router(Router::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = Client::new();

    // Health endpoints.
    let hz = client.get(format!("{base}/healthz")).send().await.unwrap();
    assert_eq!(hz.status().as_u16(), 200, "/healthz must 200");
    let rz = client.get(format!("{base}/readyz")).send().await.unwrap();
    assert_eq!(rz.status().as_u16(), 200, "/readyz must 200");

    // Content negotiation: healthz returns JSON.
    let resp = client
        .get(format!("{base}/healthz"))
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap();
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.contains("json"), "healthz must return JSON, got {ct}");
}
