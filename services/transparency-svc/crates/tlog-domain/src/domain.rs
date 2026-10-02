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

use serde::{Deserialize, Serialize};

/// Domain tag for a transparency log leaf.
///
/// Every leaf belongs to exactly one domain; the canonical log is one
/// append-only sequence, but domain tags allow filtered queries and
/// per-domain STH sub-chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    /// Voter registration events (M02/M03).
    VoterEvents,
    /// Results capture, transmission, tallying (M08/M09/M10).
    ResultsEvents,
    /// Chain of custody for election kit (M20).
    CustodyEvents,
    /// Credential issuance (kit, party agent, observer, staff).
    CredentialEvents,
    /// Meta-events: schema migrations, HSM ceremony ops, STH chains.
    MetaEvents,
}

impl Domain {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VoterEvents => "voter-events",
            Self::ResultsEvents => "results-events",
            Self::CustodyEvents => "custody-events",
            Self::CredentialEvents => "credential-events",
            Self::MetaEvents => "meta-events",
        }
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Domain {
    type Err = crate::error::DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "voter-events" => Ok(Self::VoterEvents),
            "results-events" => Ok(Self::ResultsEvents),
            "custody-events" => Ok(Self::CustodyEvents),
            "credential-events" => Ok(Self::CredentialEvents),
            "meta-events" => Ok(Self::MetaEvents),
            other => Err(crate::error::DomainError::UnknownDomain(other.to_string())),
        }
    }
}
