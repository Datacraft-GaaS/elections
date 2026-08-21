//! Diaspora voting domain, HTTP API, and transmission pipeline (WP-5.8, M24).
//!
//! Covers:
//!   - Embassy station registry (open/close lifecycle)
//!   - Diaspora roll management (import, freeze, hash)
//!   - Form 34A capture and HSM signing
//!   - Physical pouch custody chain
//!   - Delayed batch transmission with exponential backoff
//!   - CloudEvent emission via eel-outbox-rs

pub mod api;
pub mod domain;
pub mod events;
pub mod import;
pub mod roll;
pub mod transmission;
pub mod tz;

pub use domain::{
    CustodyEvent, DiasporaRoll, EmbassyStation, Form34ADiaspora, PouchCustody,
};
