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

//! In-memory store for the public training contract surface.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{
    AccreditationApplication, AssessmentSubmissionRecord, BadgeRecord, CertificateRecord,
    CompletionRecord, ContentRecord, Course, Enrolment, MedicalIncident, MedicalPersonnel,
};

/// Shared spec-state handle.
///
/// The assessment question bank lives in the crate's own [`crate::store::ExamStore`]
/// rather than here, so `submit_assessment` scores against the same registered
/// questions the exam module serves. It is held behind a lock that can be swapped
/// for a database later, as `ExamApiState` already documents.
#[derive(Default)]
pub struct SpecState {
    /// The exam store whose question bank assessments are scored against.
    pub exam: crate::handlers::exam::ExamApiState,
    pub courses: Mutex<HashMap<String, Course>>,
    pub enrolments: Mutex<HashMap<String, Enrolment>>,
    pub completions: Mutex<HashMap<String, CompletionRecord>>,
    pub applications: Mutex<HashMap<String, AccreditationApplication>>,
    pub incidents: Mutex<HashMap<String, MedicalIncident>>,
    pub personnel: Mutex<HashMap<String, MedicalPersonnel>>,
    /// Assessment submissions by `submission_id`, with the score they earned.
    pub submissions: Mutex<HashMap<String, AssessmentSubmissionRecord>>,
    /// Issued certificates by `certificate_id`.
    pub certificates: Mutex<HashMap<String, CertificateRecord>>,
    /// Issued accreditation badges by badge number.
    pub badges: Mutex<HashMap<String, BadgeRecord>>,
    /// CMS content by `content_id`.
    pub content: Mutex<HashMap<String, ContentRecord>>,
}

/// Shared spec-state handle threaded through the router.
pub type SharedSpecState = Arc<SpecState>;

impl SpecState {
    /// Construct an empty spec store.
    pub fn new() -> SharedSpecState {
        Arc::new(Self {
            exam: crate::handlers::exam::ExamApiState::new(),
            ..Default::default()
        })
    }

    /// Construct a spec store sharing an existing exam store, so the two surfaces
    /// see one question bank.
    pub fn with_exam(exam: crate::handlers::exam::ExamApiState) -> SharedSpecState {
        Arc::new(Self {
            exam,
            ..Default::default()
        })
    }

    /// An empty spec store, for callers that build the struct literally.
    pub fn empty() -> SharedSpecState {
        Self::new()
    }
}
