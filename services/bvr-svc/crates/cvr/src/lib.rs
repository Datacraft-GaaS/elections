//! Continuous voter registration — CVR ingestion and register snapshot service.
#![warn(missing_docs)]
pub mod error;
pub mod ingest;
pub mod models;
pub mod routes;
pub mod state;

use axum::Router;

/// Build the cvr router bound to the given connection pool.
pub fn app(pool: sqlx::PgPool) -> Router {
    routes::router().with_state(state::AppState::new(pool))
}
