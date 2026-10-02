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

//! Nomination aggregate — document upload, clearance lifecycle.
//! FR-M12-011, FR-M12-012, FR-M12-013, EC-DAT-07.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NominationStatus {
    Submitted,
    DocsUploaded,
    RoReviewing,
    Cleared,
    Rejected,
    Gazetted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRef {
    pub doc_id: Uuid,
    pub doc_type: String,
    pub storage_ref: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nomination {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub status: NominationStatus,
    pub documents: Vec<DocumentRef>,
    pub statutory_status: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl Nomination {
    pub fn new(id: Uuid, candidate_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            candidate_id,
            status: NominationStatus::Submitted,
            documents: Vec::new(),
            statutory_status: None,
            submitted_at: now,
            cleared_at: None,
            updated_at: now,
        }
    }

    /// Append document atomically; no partial state (EC-DAT-07).
    pub fn add_document(&mut self, doc: DocumentRef) {
        self.documents.push(doc);
        if self.status == NominationStatus::Submitted {
            self.status = NominationStatus::DocsUploaded;
        }
        self.updated_at = Utc::now();
    }
}
