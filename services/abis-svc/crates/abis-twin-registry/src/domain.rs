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

//! Domain types for twin registry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of civil-registry verification for a twin group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinVerificationStatus {
    Unverified,
    PendingCivilRegistry,
    VerifiedByCivilRegistry,
    Disputed,
}

/// A member of a twin group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinMember {
    pub subject_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub civil_registry_id: Option<String>,
}

/// A twin group record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinGroup {
    pub id: Uuid,
    pub members: Vec<TwinMember>,
    pub verification_status: TwinVerificationStatus,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl TwinGroup {
    pub fn new() -> Self {
        Self {
            id: Uuid::now_v7(),
            members: Vec::new(),
            verification_status: TwinVerificationStatus::Unverified,
            created_at: Utc::now(),
            verified_at: None,
            notes: None,
        }
    }
}

impl Default for TwinGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// A candidate twin pair detected during 1:N matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinCandidate {
    pub subject_a: Uuid,
    pub subject_b: Uuid,
    pub match_score: f64,
    pub demographic_overlap: bool,
    pub geographic_overlap: bool,
    pub detected_at: DateTime<Utc>,
}
