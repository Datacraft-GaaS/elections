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

//! SMPC protocol enum and trait.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("unsupported protocol: {0:?}")]
    Unsupported(Protocol),
    #[error("party count {count} below minimum {min}")]
    InsufficientParties { count: usize, min: usize },
}

/// Supported secure MPC protocols.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Protocol {
    /// Arithmetic secret sharing with MACs (SPDZ).
    Spdz,
    /// Information-theoretically secure, honest majority (BGW).
    Bgw,
    /// Boolean circuit garbling (GMW over binary circuits).
    Gmw,
    /// Garbled circuits (Yao, 2-party only).
    Yao,
    /// Arithmetic and boolean hybrid (AB+).
    AbPlus,
}

/// Protocol-specific minimum party count.
pub fn min_parties(protocol: &Protocol) -> usize {
    match protocol {
        Protocol::Yao => 2,
        Protocol::Spdz | Protocol::Bgw | Protocol::Gmw | Protocol::AbPlus => 3,
    }
}

/// Validates that the party count satisfies the protocol requirement.
///
/// # Errors
/// Returns `ProtocolError::InsufficientParties` when the count is too low.
pub fn validate_party_count(protocol: &Protocol, count: usize) -> Result<(), ProtocolError> {
    let min = min_parties(protocol);
    if count < min {
        return Err(ProtocolError::InsufficientParties { count, min });
    }
    Ok(())
}
