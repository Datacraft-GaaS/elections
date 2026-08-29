//! Event-sourced custody domain for M20 inventory management (WP-4.3).
#![warn(missing_docs)]
/// Domain aggregates for custody.
pub mod domain;
/// HTTP handlers for custody contract.
pub mod handlers;
/// HTTP router.
pub mod router;
/// Typed models for `custody/openapi.yaml`.
pub mod spec;
/// Postgres-backed application state (no in-memory). `Store::Mem` only behind `#[cfg(test)]`.
pub mod state;
/// Postgres + memory store (Store enum).
pub mod store;
/// Domain events.
pub mod events;
/// Geo RBAC helpers.
pub mod geo_rbac;
/// Merkle anchoring.
pub mod merkle;
/// RFID integration.
pub mod rfid;
