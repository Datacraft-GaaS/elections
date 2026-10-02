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
use crate::aggregates::form34a::{CandidateAnnotation, Form34A};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CandidateStatusAnnotation {
    Withdrawn,
    Deceased,
}

/// Annotate a candidate on a form. The candidate still tallies; the annotation
/// is preserved in the event log for audit and legal purposes (EC-LEG-01).
pub fn annotate_candidate(
    form: &mut Form34A,
    candidate_id: &str,
    annotation: CandidateStatusAnnotation,
) {
    form.candidate_annotations.retain(|a| a.candidate_id != candidate_id);
    form.candidate_annotations.push(CandidateAnnotation {
        candidate_id: candidate_id.to_string(),
        annotation,
    });
}
