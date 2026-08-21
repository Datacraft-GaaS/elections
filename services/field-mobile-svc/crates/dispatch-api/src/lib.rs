pub mod error;
pub mod models;
pub mod routes;
pub mod state;

use axum::Router;
use state::AppState;

pub fn app(state: AppState) -> Router {
	routes::router(state)
}
