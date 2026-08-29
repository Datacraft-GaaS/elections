//! OpenBao secrets client for the eel platform (ADR-0007).
//!
//! Provides:
//! - Dynamic PostgreSQL credentials via `PgDynamicCred`
//! - Transit encrypt/decrypt via `Transit`
//! - JIT admin credentials with 4h cap via `issue_admin_cred`
//! - PKI leaf cert issuance via `issue_leaf_cert`
#![warn(missing_docs)]

pub mod admin_creds;
pub mod dynamic_creds;
pub mod error;
pub mod pki;
pub mod transit;

pub use admin_creds::{AdminCred, issue_admin_cred};
pub use dynamic_creds::{PgCred, PgDynamicCred};
pub use error::SecretsError;
pub use pki::{LeafCert, issue_leaf_cert};
pub use transit::Transit;
