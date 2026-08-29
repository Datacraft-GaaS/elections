//! inspection crate.
#![warn(missing_docs)]
pub mod api;
pub mod auto_referral;
pub mod domain;
pub mod events;
pub mod search;

pub use domain::{
    AutoReferralPacket, CorrectionClaim, CorrectionStatus, CorrectionType, Determination,
    Objection, ObjectionStatus, RollSnapshot,
};
pub use search::{LookupCap, LookupResult, DAILY_CAP, FIXED_WINDOW_MS};
