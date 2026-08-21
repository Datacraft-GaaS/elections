pub mod error;
pub mod models;
pub mod routes;
pub mod state;
pub mod workflow;

use axum::Router;

pub fn app() -> Router {
    routes::router()
}
