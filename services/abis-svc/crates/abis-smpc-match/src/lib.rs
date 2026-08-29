//! eel-abis-smpc-match: Secure Multi-Party Computation biometric matching.
//!
//! Supports SPDZ, BGW, GMW, Yao, and AB+ protocols with Shamir secret sharing.
//! Tests use a 3-party local simulation.
#![warn(missing_docs)]

pub mod match_dot;
pub mod policy;
pub mod protocol;
pub mod session;
pub mod share;

pub use match_dot::{DotProductError, SecureDotProduct};
pub use policy::{SmpcPolicy, SmpcPolicyError, SmpcPolicyOutcome};
pub use protocol::{Protocol, ProtocolError};
pub use session::{SessionError, SmpcSession, SessionStatus};
pub use share::{ShareError, ShamirShare, ShamirSharing};
