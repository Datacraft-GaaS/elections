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

//! Public contract models (governance-review/post-election.yaml).

use serde::{Deserialize, Serialize};

/// A reviewer assigned to a post-election review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerAssignment {
    pub role: String,
}

/// Request body for initiating a post-election review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewInitRequest {
    pub election_id: String,
    pub scope: String,
    pub scope_code: String,
    pub reviewers: Vec<ReviewerAssignment>,
}

/// Public post-election review view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionReview {
    pub review_id: String,
    pub election_id: String,
    pub scope: String,
    pub scope_code: String,
    pub reviewers: Vec<ReviewerAssignment>,
    pub status: String,
    pub report_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for adding a review finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFindingRequest {
    pub category: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
}

/// Public review finding view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub finding_id: String,
    pub review_id: String,
    pub category: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
    pub created_at: String,
}
