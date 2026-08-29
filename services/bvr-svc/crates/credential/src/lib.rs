//! credential crate — postgres-backed (no in-memory).
#![warn(missing_docs)]
pub mod http;
pub mod spec;
/// Postgres + memory store (Store enum).
pub mod store;
