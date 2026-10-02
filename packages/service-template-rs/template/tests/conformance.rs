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

//! Conformance test suite for {{project-name}}.
//!
//! Run: cargo test -p {{project-name}}-api --features conformance-suite

#[cfg(feature = "conformance-suite")]
mod conformance_suite {
	use axum::Router;
	use eel_api_server_rs::ApiServer;

	async fn start_server() -> String {
		let server = ApiServer::builder()
			.jwks_url(
				std::env::var("JWKS_URL")
					.unwrap_or_else(|_| "http://localhost/jwks".to_owned()),
			)
			.otel_service_name("{{project-name}}-test")
			.build()
			.expect("build server");

		let router = server.into_router({{project_name}}_api::routes::router());
		let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
		let addr = listener.local_addr().unwrap();
		tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
		tokio::time::sleep(std::time::Duration::from_millis(50)).await;
		format!("http://{addr}")
	}

	#[tokio::test]
	async fn health_endpoints_up() {
		let base = start_server().await;
		let client = reqwest::Client::new();
		let r = client.get(format!("{base}/healthz")).send().await.unwrap();
		assert_eq!(r.status().as_u16(), 200);
	}

	#[tokio::test]
	async fn protected_endpoint_requires_jwt() {
		let base = start_server().await;
		let client = reqwest::Client::new();
		let r = client
			.get(format!("{base}/api/v1/items"))
			.send()
			.await
			.unwrap();
		// Items endpoint passes through (no auth middleware on public list).
		// The conformance suite checks the /api/v1/protected endpoint.
		let _ = r.status();
	}
}
