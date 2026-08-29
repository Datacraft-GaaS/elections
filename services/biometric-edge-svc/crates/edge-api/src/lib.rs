//! edge-api crate.
#![warn(missing_docs)]
pub mod error;
pub mod routes;
pub mod state;

use axum::Router;

pub fn app() -> Router {
    routes::router()
}
