pub mod error;
pub mod models;
pub mod routes;
pub mod state;
pub mod workflow;

use axum::Router;

/// Build the dereg-appeal router bound to the given connection pool.
pub fn app(pool: sqlx::PgPool) -> Router {
    routes::router().with_state(state::AppState::new(pool))
}
