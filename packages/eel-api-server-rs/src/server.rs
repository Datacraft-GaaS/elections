use std::sync::Arc;
use std::time::Instant;
use axum::{body::Body, extract::Request, middleware, middleware::Next, response::Response, Router};
use eel_auth_rs::JwksVerifier;
use eel_loadshedding_rs::{CoDelShedder, Priority};
use eel_resilience_rs::{DegradationFlag, IdempotencyStore};
use tower_http::trace::TraceLayer;

use crate::{
	builder::ApiServerBuilder,
	middleware::{
		content_negotiation::negotiate,
		degradation_guard::guard_writes,
		idempotency::idempotency_check,
		request_id::inject_request_id,
	},
	routes::health_router,
};

/// A fully-configured API server instance.
///
/// Obtain via `ApiServer::builder().build()`.
pub struct ApiServer {
	/// JWKS verifier; available to callers for `middleware::auth::require_jwt`.
	pub jwks: Arc<JwksVerifier>,
	pub(crate) idempotency: Arc<IdempotencyStore>,
	pub(crate) shedder: Arc<CoDelShedder>,
	pub(crate) degradation: Arc<DegradationFlag>,
	pub(crate) bind_addr: String,
}

async fn shed_load(
	shedder: Arc<CoDelShedder>,
	req: Request<Body>,
	next: Next,
) -> Response {
	let priority = Priority::from_headers(req.headers());
	match shedder.admit(priority, Instant::now()).await {
		Ok(_permit) => next.run(req).await,
		Err(retry_after) => {
			use axum::response::IntoResponse;
			use http::StatusCode;
			let body = serde_json::json!({
				"type": "https://eel.internal/problems/overloaded",
				"title": "Service Overloaded",
				"status": 503,
				"detail": format!("Priority class '{}' shed under load", priority.name()),
				"retry_after": retry_after,
			});
			let json = serde_json::to_string(&body).unwrap_or_default();
			(
				StatusCode::SERVICE_UNAVAILABLE,
				[
					(http::header::CONTENT_TYPE, "application/problem+json"),
					(http::header::RETRY_AFTER, &retry_after.to_string() as &str),
				],
				json,
			)
				.into_response()
		}
	}
}

impl ApiServer {
	pub fn builder() -> ApiServerBuilder {
		ApiServerBuilder::new()
	}

	/// Attach application routes and return the composed `Router`.
	///
	/// Call this to get the router for embedding in your service or for testing.
	pub fn into_router(self, app_routes: Router) -> Router {
		let idempotency = self.idempotency.clone();
		let degradation = self.degradation.clone();
		let shedder = self.shedder.clone();

		// Apply layers inside-out: the last `.layer()` call is outermost.
		app_routes
			.merge(health_router())
			.layer(middleware::from_fn(move |req, next| {
				idempotency_check(idempotency.clone(), req, next)
			}))
			.layer(middleware::from_fn(move |req, next| {
				guard_writes(degradation.clone(), req, next)
			}))
			.layer(middleware::from_fn(negotiate))
			.layer(middleware::from_fn(inject_request_id))
			.layer(TraceLayer::new_for_http())
			.layer(middleware::from_fn(move |req, next| {
				shed_load(shedder.clone(), req, next)
			}))
	}

	/// Bind and serve.
	pub async fn serve(self, app_routes: Router) -> Result<(), crate::error::ApiServerError> {
		let addr: std::net::SocketAddr = self
			.bind_addr
			.parse()
			.map_err(|e: std::net::AddrParseError| {
				crate::error::ApiServerError::Runtime(e.to_string())
			})?;

		let router = self.into_router(app_routes);
		let listener = tokio::net::TcpListener::bind(addr).await?;
		tracing::info!("eel-api-server listening on {addr}");
		axum::serve(listener, router)
			.await
			.map_err(|e| crate::error::ApiServerError::Runtime(e.to_string()))
	}
}
