//! oag crate.
#![warn(missing_docs)]
pub mod handlers;
pub mod models;
pub mod router;
pub mod state;

pub use state::{AppState, SharedState};
