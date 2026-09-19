use std::env;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::from_default_env()
				.add_directive("mobile_bff=info".parse().unwrap()),
		)
		.json()
		.init();

	let db_url = env::var("DB_URL")
		.unwrap_or_else(|_| "postgres://localhost/eel".to_string());
	let http_addr = env::var("HTTP_ADDR")
		.unwrap_or_else(|_| "0.0.0.0:8080".to_string());

	info!(db_url, http_addr, "mobile-bff starting");

	// Build PostgreSQL pool with connection pooling (10 max connections)
	let pool = sqlx::postgres::PgPoolOptions::new()
		.max_connections(10)
		.connect(&db_url)
		.await
		.expect("failed to connect to database");

	let bff = Arc::new(
		mobile_bff::spec::BffState::new(Some(pool.clone()))
			.expect("BffState init"),
	);
	let broadcast = mobile_bff::sse::ShardedBroadcast::new();

	// Full router: /verify/v1/* + /health (broadcast state) with the BFF
	// routes nested under "/" carrying their own BffState.
	let app = mobile_bff::router::build_router_with_test_state(bff, broadcast);

	let listener = tokio::net::TcpListener::bind(&http_addr)
		.await
		.expect("bind failed");

	info!(http_addr, "mobile-bff HTTP listening on http://{}", http_addr);
	axum::serve(listener, app).await.expect("HTTP server failed");
}
