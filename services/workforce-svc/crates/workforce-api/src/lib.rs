//! workforce-api — HTTP API surface for workforce-svc.
//!
//! Covers five bounded contexts from docs/workforce-lifecycle.md:
//! worker registry & contacts (§4), the recruitment state machine (§1),
//! postings & equipment claims (§5), tasking (§6) and performance (§7).

pub mod domain;
pub mod events;
pub mod handlers;
pub mod router;
pub mod store;
