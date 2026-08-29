//! Authentication and authorization primitives for the eel platform.
//!
//! - JWT verification against Keycloak JWKS with kid-miss refetch
//! - Subject extraction with roles and purpose scopes
//! - SpiceDB ReBAC check with in-memory cache and fail-closed semantics
//! - Audit event emission via the `AuditSink` trait
#![warn(missing_docs)]

pub mod error;
pub mod subject;
pub mod audit;

#[cfg(feature = "keycloak")]
pub mod jwks;

#[cfg(feature = "spicedb")]
pub mod spicedb;

pub use error::AuthError;
pub use subject::Subject;
pub use audit::{AuditEvent, AuditSink, NoopSink, LogSink};

#[cfg(feature = "keycloak")]
pub use jwks::JwksVerifier;

#[cfg(feature = "spicedb")]
pub use spicedb::{Consistency, Decision, SpiceDbClient};
