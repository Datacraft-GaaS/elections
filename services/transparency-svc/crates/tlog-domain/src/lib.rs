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

//! tlog-domain — aggregate types for the eel transparency log (WP-2.1, ADR-0017).
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod leaf;
pub mod proof;
pub mod sth;
pub mod witness;

pub use domain::Domain;
pub use error::DomainError;
pub use leaf::{Leaf, LeafFormat, LeafHash};
pub use proof::{ConsistencyProof, InclusionProof};
pub use sth::{SignedTreeHead, SthCadence, TreeSize};
pub use witness::WitnessCosignature;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_format_hash_is_deterministic() {
        use chrono::Utc;
        let t = Utc::now();
        let h1 = LeafFormat::hash(b"event", "spiffe://eel.local/svc", t, 1);
        let h2 = LeafFormat::hash(b"event", "spiffe://eel.local/svc", t, 1);
        assert_eq!(h1, h2);
    }

    #[test]
    fn domain_roundtrip() {
        use std::str::FromStr;
        let d = Domain::VoterEvents;
        let s = d.to_string();
        let d2 = Domain::from_str(&s).unwrap();
        assert_eq!(d, d2);
    }
}
