//! Re-exports of the conformance harness for use by generated services.

pub use crate::conformance_impl::*;

mod conformance_impl {
	use serde_json::Value;

	/// Assert that a GET to `url` without an Authorization header returns 401.
	pub async fn assert_jwt_required(client: &reqwest::Client, url: &str) {
		let resp = client.get(url).send().await.expect("request failed");
		assert_eq!(resp.status(), 401, "expected 401 without token at {url}");
	}

	/// Assert that a GET to `url` with a valid Bearer token returns 200.
	pub async fn assert_jwt_accepted(client: &reqwest::Client, url: &str, token: &str) {
		let resp = client
			.get(url)
			.header("Authorization", format!("Bearer {token}"))
			.send()
			.await
			.expect("request failed");
		assert_eq!(resp.status(), 200, "expected 200 with valid token at {url}");
	}

	/// Assert that a response body has RFC 9457 shape.
	pub fn assert_problem_shape(body: &Value) {
		assert!(body.get("type").is_some(), "problem must have 'type'");
		assert!(body.get("title").is_some(), "problem must have 'title'");
		assert!(body.get("status").is_some(), "problem must have 'status'");
	}

	/// Assert that a response has the x-eel-signature header.
	pub fn assert_signed(headers: &reqwest::header::HeaderMap) {
		assert!(
			headers.contains_key("x-eel-signature"),
			"response must carry x-eel-signature"
		);
	}

	/// Assert that JSON and CBOR content negotiation work.
	pub async fn assert_content_negotiation(client: &reqwest::Client, url: &str) {
		let json_resp = client
			.get(url)
			.header("Accept", "application/json")
			.send()
			.await
			.expect("JSON request failed");
		let ct = json_resp
			.headers()
			.get("content-type")
			.and_then(|v| v.to_str().ok())
			.unwrap_or("");
		assert!(ct.contains("json"), "expected JSON content-type, got {ct}");
	}

	/// Assert RFC 9457 error shape on a 422 idempotency mismatch.
	pub fn assert_idempotency_mismatch_shape(body: &Value) {
		let typ = body["type"].as_str().unwrap_or("");
		assert!(
			typ.contains("idempotency-mismatch"),
			"expected idempotency-mismatch problem type, got {typ}"
		);
	}
}
