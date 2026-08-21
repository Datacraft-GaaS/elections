pub mod error;
pub mod routes;
pub mod state;

use axum::Router;

pub fn app(state: state::AppState) -> Router {
    routes::router(state)
}
