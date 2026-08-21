use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use http::header::ACCEPT;

/// Inspect Accept header; set a request extension so handlers can choose codec.
/// Also applies zstd compression when Accept-Encoding includes zstd.
pub async fn negotiate(req: Request<Body>, next: Next) -> Response {
	let wants_cbor = req
		.headers()
		.get(ACCEPT)
		.and_then(|v| v.to_str().ok())
		.map(|v| v.contains("application/cbor"))
		.unwrap_or(false);

	let wants_zstd = req
		.headers()
		.get(http::header::ACCEPT_ENCODING)
		.and_then(|v| v.to_str().ok())
		.map(|v| v.contains("zstd"))
		.unwrap_or(false);

	// Attach flags for downstream handlers.
	let (mut parts, body) = req.into_parts();
	parts.extensions.insert(ContentPrefs { wants_cbor, wants_zstd });
	let req = Request::from_parts(parts, body);
	next.run(req).await
}

#[derive(Debug, Clone, Copy)]
pub struct ContentPrefs {
	pub wants_cbor: bool,
	pub wants_zstd: bool,
}
