//! voter-card crate.
#![warn(missing_docs)]
pub mod blacklist;
pub mod error;
pub mod models;
pub mod routes;
pub mod state;

use axum::Router;

/// Build the voter-card router bound to the given connection pool.
pub fn app(pool: sqlx::PgPool) -> Router {
    routes::router().with_state(state::AppState::new(pool))
}
