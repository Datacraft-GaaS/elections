//! BVR API — voter registration and verification service.
#![warn(missing_docs)]
pub mod error;
pub mod iprs;
pub mod nfc;
pub mod routes;
pub mod state;
pub mod store;

use axum::Router;

/// The election scope the Merkle endpoints report on.
pub const MERKLE_SCOPE: &str = "2027-general";

pub fn app(state: crate::state::AppState) -> Router {
    routes::router().with_state(state)
}
