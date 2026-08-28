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
/// In-memory application state — swap boundary: replace `Mutex<HashMap>` with `sqlx::PgPool` + `Store` trait when persistence lands.
pub mod state;
/// Domain events.
pub mod events;
/// Geo RBAC helpers.
pub mod geo_rbac;
/// Merkle anchoring.
pub mod merkle;
/// RFID integration.
pub mod rfid;
