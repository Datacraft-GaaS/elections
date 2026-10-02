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

//! In-memory exam store behind `Arc<RwLock<...>>`, matching the crate's
//! state conventions. Structured as a single aggregate so a PostgreSQL
//! implementation can slot in behind the same accessors later (§3.2).

use crate::exam::{default_blueprints, ExamBlueprint, ExamResult, ExamSession, Question};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ExamStore {
    pub blueprints: HashMap<String, ExamBlueprint>,
    pub questions: HashMap<String, Question>,
    /// Idempotent answer upserts keyed by (session_id, question_id).
    pub answers: HashMap<(String, String), Vec<usize>>,
    pub sessions: HashMap<String, ExamSession>,
    pub results: HashMap<String, ExamResult>,
}

impl ExamStore {
    /// New store seeded with the default per-role blueprints.
    pub fn new() -> Self {
        let blueprints = default_blueprints()
            .into_iter()
            .map(|b| (b.blueprint_id.clone(), b))
            .collect();
        Self {
            blueprints,
            questions: HashMap::new(),
            answers: HashMap::new(),
            sessions: HashMap::new(),
            results: HashMap::new(),
        }
    }

    /// Prior sessions for an application across ALL statuses — voided
    /// sessions included, i.e. voiding consumes an attempt (deliberate:
    /// a candidate must not be able to reset their attempt budget by
    /// abandoning a session mid-exam; §3.2 anti-abuse "one attempt chain").
    pub fn attempts_used(&self, application_id: &str) -> u32 {
        self.sessions
            .values()
            .filter(|s| s.application_id == application_id)
            .count() as u32
    }
}
