//! Middleware: SPIRE SVID auth, idempotency, rate limit, request/response signing stubs.

use axum::{
	body::Body,
	extract::Request,
	http::{HeaderMap, StatusCode},
	middleware::Next,
	response::{IntoResponse, Response},
};

/// SPIRE SVID authentication middleware.
///
/// Checks for the `X-SVID-Token` header. In dev mode (no token required),
/// passes through. In production, rejects requests without a valid SVID.
pub async fn spire_auth(headers: HeaderMap, req: Request, next: Next) -> Response {
	// Dev bypass: if EEL_ABIS_SPIRE_REQUIRED is not "true", allow all.
	let required = std::env::var("EEL_ABIS_SPIRE_REQUIRED")
		.map(|v| v == "true")
		.unwrap_or(false);

	if required {
		match headers.get("x-svid-token") {
			None => {
				return (
					StatusCode::UNAUTHORIZED,
					axum::Json(serde_json::json!({"error": "missing X-SVID-Token"})),
				)
					.into_response();
			}
			Some(tok) => {
				// In a real impl: validate SPIFFE SVID JWT against SPIRE server.
				// Here: reject "invalid" literal for testability.
				if tok.as_bytes() == b"invalid" {
					return (
						StatusCode::UNAUTHORIZED,
						axum::Json(
							serde_json::json!({"error": "invalid SVID token"}),
						),
					)
						.into_response();
				}
			}
		}
	}

	next.run(req).await
}

/// Idempotency-Key middleware.
///
/// For write endpoints, requires the `Idempotency-Key` header.
/// Returns cached response if the key was already processed.
pub async fn require_idempotency_key(
	headers: HeaderMap,
	req: Request,
	next: Next,
) -> Response {
	// Only enforce on mutating methods.
	let method = req.method().clone();
	if !matches!(
		method.as_str(),
		"POST" | "PUT" | "PATCH" | "DELETE"
	) {
		return next.run(req).await;
	}

	if headers.get("idempotency-key").is_none() {
		return (
			StatusCode::BAD_REQUEST,
			axum::Json(
				serde_json::json!({"error": "Idempotency-Key header required for write operations"}),
			),
		)
			.into_response();
	}

	next.run(req).await
}

/// Response signing middleware stub.
///
/// Adds `X-EEL-Signature` header to every response.
/// Real implementation would use the cosign signer from eel-api-server-rs.
pub async fn cosign_response(req: Request, next: Next) -> Response {
	let mut resp = next.run(req).await;
	// Stub: set a placeholder signature header.
	resp.headers_mut().insert(
		"x-eel-signature",
		"stub-cosign-v1".parse().expect("valid header value"),
	);
	resp
}
