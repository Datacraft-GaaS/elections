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

use crate::domain::Domain;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TreeSize = u64;

/// STH publication cadence per ADR-0017.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SthCadence {
    /// 1-second hot STH.
    Hot,
    /// 60-second default STH.
    Default,
    /// 5-minute batch STH (used as external witness anchor).
    Batch,
}

impl SthCadence {
    pub fn interval_secs(self) -> u64 {
        match self {
            Self::Hot => 1,
            Self::Default => 60,
            Self::Batch => 300,
        }
    }
}

/// A Signed Tree Head.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTreeHead {
    pub id: Uuid,
    /// Number of leaves in the tree at this point.
    pub tree_size: TreeSize,
    /// BLAKE3 root hash (hex-encoded).
    pub root_hash: String,
    /// Domain this STH covers (None = all domains / super-root).
    pub domain: Option<Domain>,
    /// Cadence at which this STH was produced.
    pub cadence: SthCadence,
    /// Wall-clock time the STH was signed.
    pub signed_at: DateTime<Utc>,
    /// HSM signature over `tree_size || root_hash || signed_at` (hex-encoded).
    pub signature: String,
    /// Key role label (e.g. "sth").
    pub key_role: String,
    /// Fingerprint of the HSM key that signed this STH.
    pub key_fingerprint: String,
}
