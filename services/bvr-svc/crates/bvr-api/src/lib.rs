//! BVR API — voter registration and verification service.
#![warn(missing_docs)]
pub mod error;
pub mod iprs;
pub mod nfc;
pub mod routes;
pub mod state;

use axum::Router;

pub fn app() -> Router {
    routes::router()
}
