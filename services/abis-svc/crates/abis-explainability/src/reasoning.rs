use crate::ExplainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityResult {
    pub modality: String,
    pub matched_minutiae: u32,
    pub probe_total: u32,
    pub gallery_total: u32,
    pub fusion_score: f32,
    pub threshold: f32,
}

pub fn generate_reasoning_text(
    results: &[ModalityResult],
    overall_decision: bool,
) -> Result<String, ExplainError> {
    if results.is_empty() {
        return Err(ExplainError::EmptyTemplate);
    }

    let mut lines = Vec::new();
    for r in results {
        let decision = if r.fusion_score >= r.threshold {
            "MATCH"
        } else {
            "NO MATCH"
        };
        let verb = if r.fusion_score >= r.threshold {
            "exceeds"
        } else {
            "is below"
        };
        lines.push(format!(
            "{}: {} minutiae matched of {} in probe; {} of {} in gallery. Fusion score {:.2} {} threshold {:.2}. {}.",
            r.modality,
            r.matched_minutiae,
            r.probe_total,
            r.matched_minutiae,
            r.gallery_total,
            r.fusion_score,
            verb,
            r.threshold,
            decision,
        ));
    }

    let overall = if overall_decision {
        "MATCH"
    } else {
        "NO MATCH"
    };
    lines.push(format!("Overall decision: {}.", overall));

    Ok(lines.join("\n"))
}
