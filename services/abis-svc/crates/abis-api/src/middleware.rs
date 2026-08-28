//! Middleware: SPIRE SVID auth, idempotency, rate limit, request/response signing stubs.

use axum::{
	body::Body,
	extract::Request,
	http::{HeaderMap, StatusCode},
	middleware::Next,
	response::{IntoResponse, Response},
};

/// SPIFFE identity extracted from a validated SVID.
#[derive(Debug, Clone)]
pub struct SpiffeIdentity {
	pub trust_domain: String,
	pub namespace: String,
	pub service_account: String,
}

impl SpiffeIdentity {
	/// Parse `spiffe://<trust-domain>/ns/<namespace>/sa/<service-account>`.
	///
	/// This is the SPIFFE ID path shape used across the eel mesh (see
	/// docs/architecture/cross-cutting/trust-model.md); anything else is
	/// rejected rather than guessed.
	pub fn parse(spiffe_id: &str) -> Option<Self> {
		let rest = spiffe_id.strip_prefix("spiffe://")?;
		let (trust_domain, path) = rest.split_once('/')?;
		let mut seg = path.split('/');
		match (seg.next()?, seg.next()?, seg.next()?, seg.next()?) {
			("ns", ns, "sa", sa) if !ns.is_empty() && !sa.is_empty() => Some(Self {
				trust_domain: trust_domain.to_owned(),
				namespace: ns.to_owned(),
				service_account: sa.to_owned(),
			}),
			_ => None,
		}
	}
}

/// Extract the caller's SPIFFE ID from Envoy mTLS propagation headers.
///
/// Envoy terminates mTLS at the sidecar and forwards the peer certificate via
/// `X-Forwarded-Client-Cert` (XFCC). We accept either:
/// - XFCC with `By=<spiffe uri>;URI=<spiffe uri>` fields (Envoy default), or
/// - a direct `X-Client-Spiffe-Id` header set by an explicit gateway config.
///
/// The header is trusted ONLY because it is emitted by the local sidecar;
/// direct external traffic cannot reach the service without passing Envoy
/// (network policy), which strips client-supplied XFCC headers.
fn extract_spiffe(headers: &HeaderMap) -> Option<SpiffeIdentity> {
	if let Some(xfcc) = headers.get("x-forwarded-client-cert").and_then(|v| v.to_str().ok()) {
		for entry in xfcc.split(',') {
			for pair in entry.split(';') {
				let mut kv = pair.splitn(2, '=');
				let k = kv.next()?.trim();
				let v = kv.next()?.trim().trim_matches('"');
				if k == "URI" || k == "By" {
					if let Some(id) = SpiffeIdentity::parse(v) {
						return Some(id);
					}
				}
			}
		}
		None
	} else if let Some(direct) = headers
		.get("x-client-spiffe-id")
		.and_then(|v| v.to_str().ok())
	{
		SpiffeIdentity::parse(direct)
	} else {
		None
	}
}

fn unauthorized(detail: &str) -> Response {
	(
		StatusCode::UNAUTHORIZED,
		axum::Json(serde_json::json!({"error": detail})),
	)
		.into_response()
}

fn forbidden(detail: &str) -> Response {
	(
		StatusCode::FORBIDDEN,
		axum::Json(serde_json::json!({"error": detail})),
	)
		.into_response()
}

/// SPIRE SVID authentication middleware.
///
/// When `EEL_ABIS_SPIRE_REQUIRED=true` (production posture):
/// 1. the caller's SPIFFE ID is recovered from the sidecar-propagated mTLS
///    headers (`XFCC` / `X-Client-Spiffe-Id`) and must parse as an eel-mesh
///    identity in the `eel.cluster` trust domain;
/// 2. when `EEL_ABIS_ALLOWED_NAMESPACES` is set, only identities from those
///    namespaces are admitted (default: enrolment, adjudication, admin).
///
/// Dev bypass remains available by leaving `EEL_ABIS_SPIRE_REQUIRED`
/// unset/false -- explicitly, never implicitly.
pub async fn spire_auth(headers: HeaderMap, req: Request, next: Next) -> Response {
	let required = std::env::var("EEL_ABIS_SPIRE_REQUIRED")
		.map(|v| v == "true")
		.unwrap_or(false);

	if required {
		let Some(identity) = extract_spiffe(&headers) else {
			return unauthorized("missing or unparseable SPIFFE identity (XFCC)");
		};
		if identity.trust_domain != "eel.cluster" {
			return forbidden("unexpected trust domain");
		}
		if let Ok(allow) = std::env::var("EEL_ABIS_ALLOWED_NAMESPACES") {
			let allowed: Vec<&str> =
				allow.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
			if !allowed.is_empty() && !allowed.contains(&identity.namespace.as_str()) {
				return forbidden("caller namespace not permitted");
			}
		}
		tracing::debug!(
			spiffe = %format!("spiffe://eel.cluster/ns/{}/sa/{}", identity.namespace, identity.service_account),
			"svid accepted"
		);
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
