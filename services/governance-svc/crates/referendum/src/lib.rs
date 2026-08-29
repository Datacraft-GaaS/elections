//! referendum crate.
#![warn(missing_docs)]
pub mod counties;
pub mod explainer;
pub mod events;
pub mod handlers;
pub mod models;
pub mod router;
pub mod state;
pub mod threshold;

pub use state::{AppState, SharedState};
