//! AI platform API library: store contract and shared types.
//!
//! The HTTP surface (handlers/router) is being built out; the persistence
//! contract below is complete with Pg and in-memory implementations.

pub mod store;

pub use store::{AiStore, StoreError};
