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

//! Public contract models (governance-by-election/operations.yaml).

use serde::{Deserialize, Serialize};

/// Request body for declaring a by-election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByElectionDeclaration {
    pub trigger_reason: String,
    pub constituency_code: String,
    pub gazette_ref: String,
    pub election_date: String,
}

/// Public by-election view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByElection {
    pub by_election_id: String,
    pub trigger_reason: String,
    pub constituency_code: String,
    pub gazette_ref: String,
    pub election_date: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for a status transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTransition {
    pub new_status: String,
    pub reason: String,
}
