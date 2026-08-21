//! Conformance test for the service-template-rs package itself.
//!
//! Spins up an `ApiServer` with no app routes and verifies the shared suite passes.

mod conformance;

#[tokio::test]
async fn conformance_suite_passes_against_bare_server() {
	use axum::Router;
	use eel_api_server_rs::ApiServer;

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

	// Give the server a moment to start.
	tokio::time::sleep(std::time::Duration::from_millis(50)).await;

	conformance::run_health_suite(&base).await;
	conformance::run_content_negotiation_suite(&base).await;
	// Auth suite skipped: no /api/v1/protected route on the bare server.
}
