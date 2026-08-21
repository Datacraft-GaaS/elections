use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use eel_resilience_rs::DegradationFlag;
use crate::problem::Problem;

/// Reject mutating requests (POST/PUT/PATCH/DELETE) when in ReadOnlyLockdown.
pub async fn guard_writes(
	flag: std::sync::Arc<DegradationFlag>,
	req: Request<Body>,
	next: Next,
) -> Response {
	use http::Method;
	let method = req.method().clone();
	let is_write = matches!(method, Method::POST | Method::PUT | Method::PATCH | Method::DELETE);
	if is_write && !flag.is_write_allowed().await {
		return Problem::write_locked().to_axum_response();
	}
	next.run(req).await
}
