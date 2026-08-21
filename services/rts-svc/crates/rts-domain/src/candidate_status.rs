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
