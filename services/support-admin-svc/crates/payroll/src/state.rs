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

//! Shared in-memory state for the payroll service (WP-5.6).
//!
//! The scaffolding phase keeps document submissions in-process; production
//! will swap this for sqlx-backed repositories once persistence lands.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::domain::{DocumentSubmission, JobApplication, Verdict};

#[derive(Debug, Default)]
struct DocumentStoreInner {
    submissions: HashMap<Uuid, DocumentSubmission>,
    /// Idempotency index over `(application_id, doc_type, content_hash_hex)`.
    idempotency: HashMap<(Uuid, String, String), Uuid>,
}

/// In-process document store with idempotent inserts.
#[derive(Debug, Default)]
pub struct DocumentStore {
    inner: RwLock<DocumentStoreInner>,
}

impl DocumentStore {
    /// Existing submission id for `(application_id, doc_type, content_hash)`,
    /// if any — a `Some` means the insert is a replay.
    pub fn find_idempotent(
        &self,
        application_id: Uuid,
        doc_type: &str,
        content_hash_hex: &str,
    ) -> Option<Uuid> {
        self.inner
            .read()
            .expect("document store lock poisoned")
            .idempotency
            .get(&(
                application_id,
                doc_type.to_string(),
                content_hash_hex.to_string(),
            ))
            .copied()
    }

    /// Registers a new submission and its idempotency key.
    pub fn insert(&self, submission: DocumentSubmission) {
        let mut inner = self.inner.write().expect("document store lock poisoned");
        inner.idempotency.insert(
            (
                submission.application_id,
                submission.doc_type.clone(),
                submission.content_hash_hex.clone(),
            ),
            submission.id,
        );
        inner.submissions.insert(submission.id, submission);
    }

    pub fn get(&self, id: Uuid) -> Option<DocumentSubmission> {
        self.inner
            .read()
            .expect("document store lock poisoned")
            .submissions
            .get(&id)
            .cloned()
    }

    pub fn all(&self) -> Vec<DocumentSubmission> {
        self.inner
            .read()
            .expect("document store lock poisoned")
            .submissions
            .values()
            .cloned()
            .collect()
    }

    pub fn for_application(&self, application_id: Uuid) -> Vec<DocumentSubmission> {
        self.all()
            .into_iter()
            .filter(|s| s.application_id == application_id)
            .collect()
    }

    /// Marks `old_id` as superseded by `new_id`; returns false if missing or
    /// already superseded.
    pub fn mark_superseded(&self, old_id: Uuid, new_id: Uuid) -> bool {
        let mut inner = self.inner.write().expect("document store lock poisoned");
        match inner.submissions.get_mut(&old_id) {
            Some(submission) if submission.superseded_by.is_none() => {
                submission.superseded_by = Some(new_id);
                true
            }
            _ => false,
        }
    }

    /// Applies a verdict to a pending submission; returns false when the
    /// submission is missing or already decided.
    pub fn apply_verdict(&self, id: Uuid, verdict: Verdict) -> bool {
        let mut inner = self.inner.write().expect("document store lock poisoned");
        match inner.submissions.get_mut(&id) {
            Some(submission) if submission.verdict == Verdict::Pending => {
                submission.verdict = verdict;
                true
            }
            _ => false,
        }
    }
}

/// Application state shared across handlers.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub documents: Arc<DocumentStore>,
    pub applications: Arc<ApplicationStore>,
}

// ---------------------------------------------------------------------------
// Application intake store
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct ApplicationStoreInner {
    applications: HashMap<Uuid, JobApplication>,
    /// Reference string -> application id.
    by_ref: HashMap<String, Uuid>,
    /// National-ID hash -> application id (idempotent resubmission).
    by_national_id: HashMap<String, Uuid>,
}

/// In-process job-application store (idempotent on national-ID hash).
#[derive(Debug, Default)]
pub struct ApplicationStore {
    inner: RwLock<ApplicationStoreInner>,
}

impl ApplicationStore {
    /// Existing application for a national-ID hash, if any — a `Some` means
    /// the submission is a replay and should return the original record.
    pub fn find_by_national_id(&self, national_id_hash: &str) -> Option<JobApplication> {
        let inner = self.inner.read().expect("application store lock poisoned");
        inner
            .by_national_id
            .get(national_id_hash)
            .and_then(|id| inner.applications.get(id))
            .cloned()
    }

    pub fn insert(&self, application: JobApplication) {
        let mut inner = self.inner.write().expect("application store lock poisoned");
        inner
            .by_ref
            .insert(application.application_ref.clone(), application.id);
        inner
            .by_national_id
            .insert(application.national_id_hash.clone(), application.id);
        inner.applications.insert(application.id, application);
    }

    pub fn get(&self, id: Uuid) -> Option<JobApplication> {
        self.inner
            .read()
            .expect("application store lock poisoned")
            .applications
            .get(&id)
            .cloned()
    }

    pub fn find_by_ref(&self, reference: &str) -> Option<JobApplication> {
        let inner = self.inner.read().expect("application store lock poisoned");
        inner
            .by_ref
            .get(reference)
            .and_then(|id| inner.applications.get(id))
            .cloned()
    }
}
