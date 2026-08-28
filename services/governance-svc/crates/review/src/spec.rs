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
