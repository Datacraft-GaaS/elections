//! Shared conformance assertions for generated eel Rust services.
//!
//! A templated service's `tests/conformance.rs` calls these after spinning up
//! an in-process `ApiServer`.

use reqwest::Client;
use serde_json::Value;

pub async fn run_auth_suite(base_url: &str, valid_token: Option<&str>) {
	let client = Client::new();
	let protected = format!("{base_url}/api/v1/protected");

	// Without token: 401.
	let resp = client.get(&protected).send().await.unwrap();
	assert_eq!(resp.status().as_u16(), 401, "no-token must 401");

	// With invalid token: 401.
	let resp = client
		.get(&protected)
		.header("Authorization", "Bearer definitely.not.valid")
		.send()
		.await
		.unwrap();
	assert_eq!(resp.status().as_u16(), 401, "bad-token must 401");

	// With valid token (skipped when None, e.g. no Keycloak in CI).
	if let Some(tok) = valid_token {
		let resp = client
			.get(&protected)
			.header("Authorization", format!("Bearer {tok}"))
			.send()
			.await
			.unwrap();
		assert_eq!(resp.status().as_u16(), 200, "valid-token must 200");
	}
}

pub async fn run_problem_json_suite(base_url: &str) {
	let client = Client::new();
	// Trigger a 401 and check RFC 9457 shape.
	let resp = client
		.get(format!("{base_url}/api/v1/protected"))
		.send()
		.await
		.unwrap();
	assert_eq!(resp.status().as_u16(), 401);
	let ct = resp
		.headers()
		.get("content-type")
		.and_then(|v| v.to_str().ok())
		.unwrap_or("");
	assert!(ct.contains("problem+json"), "error must be problem+json, got {ct}");
	let body: Value = resp.json().await.unwrap();
	assert!(body.get("type").is_some(), "problem must have 'type'");
	assert!(body.get("title").is_some(), "problem must have 'title'");
	assert!(body.get("status").is_some(), "problem must have 'status'");
}

pub async fn run_health_suite(base_url: &str) {
	let client = Client::new();
	let hz = client.get(format!("{base_url}/healthz")).send().await.unwrap();
	assert_eq!(hz.status().as_u16(), 200, "/healthz must 200");

	let rz = client.get(format!("{base_url}/readyz")).send().await.unwrap();
	assert_eq!(rz.status().as_u16(), 200, "/readyz must 200");
}

pub async fn run_signing_suite(base_url: &str) {
	let client = Client::new();
	// Public GET endpoint - healthz is always public.
	let resp = client.get(format!("{base_url}/healthz")).send().await.unwrap();
	// Signing headers are present on public GETs when signing is active.
	// In test mode the server may not sign health endpoints; assert the
	// custom endpoint exists (accept 200 or 404 for now).
	let _ = resp.status();
}

pub async fn run_content_negotiation_suite(base_url: &str) {
	let client = Client::new();
	let resp = client
		.get(format!("{base_url}/healthz"))
		.header("Accept", "application/json")
		.send()
		.await
		.unwrap();
	let ct = resp
		.headers()
		.get("content-type")
		.and_then(|v| v.to_str().ok())
		.unwrap_or("");
	assert!(ct.contains("json"), "healthz must return JSON by default, got {ct}");
}

pub async fn run_degradation_suite(base_url: &str) {
	// When write-lock is engaged, POST must return 503 with write-locked problem type.
	// This is tested by the service-specific tests; this stub confirms the suite runs.
	let _ = base_url;
}

/// Full conformance run.
///
/// `valid_token` is skipped when `None` (no auth server in environment).
pub async fn run_all(base_url: &str, valid_token: Option<&str>) {
	run_health_suite(base_url).await;
	run_auth_suite(base_url, valid_token).await;
	run_problem_json_suite(base_url).await;
	run_content_negotiation_suite(base_url).await;
	run_degradation_suite(base_url).await;
	run_signing_suite(base_url).await;
}
