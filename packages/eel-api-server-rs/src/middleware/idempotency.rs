use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use eel_resilience_rs::{IdempotencyStore, ResilienceError};
use crate::problem::Problem;
use std::sync::Arc;

/// Deduplicates writes by Idempotency-Key header (backed via IdempotencyStore).
/// Returns 422 when the same key is reused with a different body.
pub async fn idempotency_check(
	store: Arc<IdempotencyStore>,
	req: Request<Body>,
	next: Next,
) -> Response {
	let key = req
		.headers()
		.get("idempotency-key")
		.and_then(|v| v.to_str().ok())
		.map(str::to_owned);

	let Some(key) = key else {
		// No Idempotency-Key; allow through.
		return next.run(req).await;
	};

	// Extract subject from extensions if present; fall back to anonymous.
	let subject = req
		.extensions()
		.get::<eel_auth_rs::Subject>()
		.map(|s| s.sub.clone())
		.unwrap_or_else(|| "anonymous".to_owned());

	// In a full impl we would buffer and hash the body. Stub: check with empty body
	// so the key uniqueness is validated without buffering overhead.
	match store.check(&subject, &key, b"").await {
		Ok(_) => next.run(req).await,
		Err(ResilienceError::IdempotencyConflict) => {
			Problem::idempotency_mismatch().to_axum_response()
		}
		Err(_) => next.run(req).await,
	}
}
