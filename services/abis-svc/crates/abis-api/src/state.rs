//! Application state shared across handlers.

/// Shared application state.
///
/// The PostgreSQL pool is constructed lazily so the router can be built
/// (and exercised in hermetic tests) without a live database; connections
/// are attempted only when a handler touches the store. Store failures
/// surface as 503 problem+json responses, never at construction time.
pub struct AppState {
	pub db: sqlx::PgPool,
}

impl AppState {
	/// Build state from the environment (`EEL_ABIS_DATABASE_URL`, falling
	/// back to `DATABASE_URL`, then to a local default).
	pub fn new() -> Self {
		let url = std::env::var("EEL_ABIS_DATABASE_URL")
			.or_else(|_| std::env::var("DATABASE_URL"))
			.unwrap_or_else(|_| "postgres://127.0.0.1:5432/eel_abis".to_string());
		Self::from_url(&url)
	}

	pub fn from_url(url: &str) -> Self {
		let db = sqlx::postgres::PgPoolOptions::new()
			.max_connections(16)
			.acquire_timeout(std::time::Duration::from_secs(5))
			.connect_lazy(url)
			.expect("connect_lazy cannot fail");
		Self { db }
	}
}

impl Default for AppState {
	fn default() -> Self {
		Self::new()
	}
}
