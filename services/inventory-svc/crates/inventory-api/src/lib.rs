//! inventory-api — HTTP API surface for inventory-svc.
//!
//! Exposes the custody (M20), transport & security, and warehouse/pallet/
//! incident domains over REST. [`store::Store`] is an enum over Postgres
//! (sqlx) and an in-memory fake, so the service runs without a database and
//! is fully testable. [`auth::Auth`] adds JWT + SpiceDB authorization when
//! configured via the environment.
#![warn(missing_docs)]

pub mod auth;
pub mod handlers;
pub mod router;
pub mod spec;
pub mod store;
