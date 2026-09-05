use std::env;
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
	let pool = sqlx::PgPoolOptions::new()
		.max_connections(10)
		.connect(&db_url)
		.await
		.expect("failed to connect to database");

	let state = Arc::new(mobile_bff::spec::BffState::new(pool.clone()).unwrap());

	// Build the router with all endpoints wired to real handlers
	let app = mobile_bff::spec::router(state);

	// Attach health check endpoint at root
	axum::Router::new()
		.route("/healthz", axum::routing::get(|| async {"ok"}))
		.merge(app)
		.with_state(state.clone())
		.into_make_service();

	let listener = tokio::net::TcpListener::bind(&http_addr)
		.await
		.expect("bind failed");

	info!(http_addr, "mobile-bff HTTP listening on http://{}", http_addr);
	axum::serve(listener, app).await.expect("HTTP server failed");
}
