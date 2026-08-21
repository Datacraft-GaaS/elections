pub mod error;
pub mod ingest;
pub mod models;
pub mod routes;
pub mod state;

use axum::Router;

pub fn app() -> Router {
    routes::router()
}
