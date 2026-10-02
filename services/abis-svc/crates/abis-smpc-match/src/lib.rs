// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
