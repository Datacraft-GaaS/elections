//! mobile-bff crate — BFF gateway for field mobile apps (M13, M29, M30, M31, etc.)
#![warn(missing_docs)]
pub mod handlers;
pub mod mobile_store;
pub mod models;
pub mod router;
pub mod spec;
pub mod sse;
pub mod verify_store;
