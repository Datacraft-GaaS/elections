//! `eel-inventory-physical-seal-binding` (HARD-16)
//!
//! Cryptographic binding of a physical seal photo to a custody-chain event.
//! The kit HSM signs a SHA-256 hash of the photo bytes bound to the seal serial,
//! then this crate verifies the signature before a CustodyEvent is accepted.

pub mod api;
pub mod bind;
pub mod error;
pub mod seal_photo;
pub mod verify;

pub use bind::{bind_custody_event, BoundCustodyEvent};
pub use error::SealBindError;
pub use seal_photo::SealPhotoSignature;
pub use verify::verify_bound_event;
