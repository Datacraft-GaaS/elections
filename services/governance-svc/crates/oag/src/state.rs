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

//! In-memory store for the OAG engagement surface.
//!
//! Previously the crate had **no state module at all**. `list_engagements` and
//! `list_published_reports` returned `vec![]`, `create_engagement` and
//! `register_published_report` returned `T::default()`, `get_engagement` echoed
//! the path id into a default struct, and `extract_evidence` /
//! `record_response` did the same. So a client could read an engagement that had
//! never been created, and a response could be recorded against one that did not
//! exist.
//!
//! The lifecycle an engagement follows is `Scoping -> Active -> Responded ->
//! Closed`. Evidence may be extracted from an engagement that exists; a response
//! closes the question and moves the engagement to `Responded`, after which no
//! further response may be recorded. See `handlers.rs` for which transitions the
//! handlers allow.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{EvidenceExtract, OagEngagement, OagResponse, PublishedReport};

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Engagements by `engagement_id`.
    pub engagements: Mutex<HashMap<String, OagEngagement>>,
    /// Evidence extracts by `extract_id`.
    pub extracts: Mutex<HashMap<String, EvidenceExtract>>,
    /// Responses by `response_id`.
    pub responses: Mutex<HashMap<String, OagResponse>>,
    /// Published reports by `pub_report_id`.
    pub published_reports: Mutex<HashMap<String, PublishedReport>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }

    /// All engagements, newest first.
    pub fn engagements_sorted(&self) -> Vec<OagEngagement> {
        let guard = self.engagements.lock().expect("lock poisoned");
        let mut items: Vec<OagEngagement> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| b.initiated_at.cmp(&a.initiated_at).then(b.engagement_id.cmp(&a.engagement_id)));
        items
    }

    /// Evidence extracts for an engagement, oldest first.
    pub fn extracts_for(&self, engagement_id: &str) -> Vec<EvidenceExtract> {
        let guard = self.extracts.lock().expect("lock poisoned");
        let mut items: Vec<EvidenceExtract> = guard
            .values()
            .filter(|e| e.engagement_id == engagement_id)
            .cloned()
            .collect();
        drop(guard);
        items.sort_by(|a, b| a.extracted_at.cmp(&b.extracted_at).then(a.extract_id.cmp(&b.extract_id)));
        items
    }

    /// All published reports, newest first.
    pub fn published_reports_sorted(&self) -> Vec<PublishedReport> {
        let guard = self.published_reports.lock().expect("lock poisoned");
        let mut items: Vec<PublishedReport> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| b.published_at.cmp(&a.published_at).then(a.pub_report_id.cmp(&b.pub_report_id)));
        items
    }
}
