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

/// An audit engagement opened by the OAG.
/// A request to open an engagement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateEngagementRequest {
    pub scope: String,
    pub oag_ref: String,
}

/// A request to extract evidence.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractEvidenceRequest {
    pub description: String,
    pub extracted_by: String,
}

/// A request to record the OAG's response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordResponseRequest {
    pub summary: String,
    pub disposition: String,
    pub oag_signatory: String,
}

/// A request to register a published report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegisterPublishedReportRequest {
    pub title: String,
    pub report_type: String,
    pub uri: String,
    /// The digest of the published report's contents, when the caller holds it.
    #[serde(default)]
    pub hash: Option<String>,
}

/// An audit engagement opened by the OAG.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OagEngagement {
    pub engagement_id: String,
    pub scope: String,
    pub status: EngagementStatus,
    pub initiated_at: String,
    pub oag_ref: String,
}

/// Where an engagement sits in its lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EngagementStatus {
    /// Opened, scope not yet agreed.
    #[default]
    Scoping,
    /// In progress; evidence may be extracted.
    Active,
    /// The OAG has responded; no further response may be recorded.
    Responded,
    /// Concluded.
    Closed,
}

impl EngagementStatus {
    /// The wire value, derived from serde so it cannot drift from the enum.
    pub fn wire(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
    }

    /// Whether evidence may still be extracted.
    pub fn accepts_evidence(&self) -> bool {
        matches!(self, Self::Scoping | Self::Active)
    }

    /// Whether a response may still be recorded.
    ///
    /// False once responded: the OAG has answered, and a second answer would
    /// silently replace the first.
    pub fn accepts_response(&self) -> bool {
        matches!(self, Self::Scoping | Self::Active)
    }
}

/// Evidence extracted from an engagement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceExtract {
    pub extract_id: String,
    pub engagement_id: String,
    pub description: String,
    pub payload_hash: String,
    pub extracted_at: String,
    pub extracted_by: String,
}

/// The OAG's response to an engagement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OagResponse {
    pub response_id: String,
    pub engagement_id: String,
    pub summary: String,
    pub disposition: String,
    pub responded_at: String,
    pub oag_signatory: String,
}

/// A report the OAG has published.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublishedReport {
    pub pub_report_id: String,
    pub title: String,
    pub report_type: String,
    pub published_at: String,
    pub uri: String,
    /// Null when the caller did not supply one. This service does not hold the
    /// report contents, so it cannot compute this itself, and a digest of the
    /// metadata would look like a content hash without being one.
    pub hash: Option<String>,
}
