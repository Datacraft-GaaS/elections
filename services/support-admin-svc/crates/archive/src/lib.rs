//! `archive` HTTP service for M69 long-term-archive (WP-5.6).
//!
//! Manages archival collections, ingest jobs, preservation attestations,
//! research access requests, destruction certificates, and citation records
//! for support-admin-svc.
//!
//! WORM storage: MinIO Object Lock is configured via bucket policy (s3:PutObjectLegalHold,
//! COMPLIANCE mode, 15-year retention). This is enforced at the storage layer and is not
//! modeled in this domain.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod handlers;
pub mod routes;
