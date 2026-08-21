//! Enrolment pipeline: pure state-machine stages for the enrolment lifecycle.
//!
//! Each stage accepts immutable inputs and returns the next pipeline state plus
//! any CloudEvent envelopes to publish. Actual matcher I/O (quality gate, 1:N
//! search) is injected by the orchestrator; the pipeline only drives transitions.

use crate::domain::{
    DedupDecision, DedupResult, EnrolmentDecision, EnrolmentJob, EnrolmentStatus, QualityReport,
};
use crate::error::EnrolmentError;
use crate::events;

/// Pipeline state: carries the job and intermediate results between stages.
#[derive(Debug, Clone)]
pub struct PipelineState {
    pub job: EnrolmentJob,
    pub quality_report: Option<QualityReport>,
    pub dedup_result: Option<DedupResult>,
}

impl PipelineState {
    pub fn new(job: EnrolmentJob) -> Self {
        Self {
            job,
            quality_report: None,
            dedup_result: None,
        }
    }
}

/// Stage 1: record receipt, set status to Received, emit envelope.
pub fn ingest(
    job: EnrolmentJob,
) -> Result<(PipelineState, eel_events_rs::envelope::Envelope), EnrolmentError> {
    let env = events::enrolment_received_event(&job)
        .map_err(|e| EnrolmentError::EventBuild(e.to_string()))?;
    Ok((PipelineState::new(job), env))
}

/// Stage 2: apply a completed quality report to the state.
///
/// On pass: advances to DedupPending.
/// On fail: sets status to Rejected and returns the rejection event.
pub fn apply_quality_report(
    mut state: PipelineState,
    report: QualityReport,
) -> Result<(PipelineState, Vec<eel_events_rs::envelope::Envelope>), EnrolmentError> {
    let mut events_out = vec![];
    state.job.status = EnrolmentStatus::QualityChecking;

    if report.is_acceptable() {
        state.job.status = EnrolmentStatus::DedupPending;
        state.job.quality_report = Some(report.clone());
        state.quality_report = Some(report);
    } else {
        let reasons = report.retake_reasons.clone();
        state.job.status = EnrolmentStatus::Rejected;
        state.job.quality_report = Some(report.clone());
        let ev = events::enrolment_rejected_event(&state.job, &reasons)
            .map_err(|e| EnrolmentError::EventBuild(e.to_string()))?;
        events_out.push(ev);
        state.quality_report = Some(report);
    }

    Ok((state, events_out))
}

/// Stage 3: apply a completed dedup result to the state.
///
/// Advances status through DedupInProgress -> DedupComplete.
pub fn apply_dedup_result(mut state: PipelineState, result: DedupResult) -> PipelineState {
    state.job.status = EnrolmentStatus::DedupInProgress;
    state.job.status = EnrolmentStatus::DedupComplete;
    state.dedup_result = Some(result);
    state
}

/// Stage 4: route based on dedup decision.
///
/// NoMatch -> Enrolled + enrolled event.
/// HighConfidenceMatch or ManualAdjudicationRequired -> hold + dedup_hold event.
pub fn route_decision(
    mut state: PipelineState,
) -> Result<(PipelineState, Vec<eel_events_rs::envelope::Envelope>), EnrolmentError> {
    let dedup = state
        .dedup_result
        .as_ref()
        .ok_or(EnrolmentError::MissingDedupResult)?
        .clone();

    let mut events_out = vec![];

    match &dedup.decision {
        DedupDecision::NoMatch => {
            state.job.status = EnrolmentStatus::Enrolled;
            let ev =
                events::enrolled_event(state.job.id, state.job.subject_id, "shard-auto-assigned")
                    .map_err(|e| EnrolmentError::EventBuild(e.to_string()))?;
            events_out.push(ev);
        }
        DedupDecision::HighConfidenceMatch => {
            state.job.status = EnrolmentStatus::DedupHold;
            let ev = events::dedup_hold_event(state.job.id, state.job.subject_id, &dedup.decision)
                .map_err(|e| EnrolmentError::EventBuild(e.to_string()))?;
            events_out.push(ev);
        }
        DedupDecision::ManualAdjudicationRequired => {
            state.job.status = EnrolmentStatus::AdjudicationRequired;
            let ev = events::dedup_hold_event(state.job.id, state.job.subject_id, &dedup.decision)
                .map_err(|e| EnrolmentError::EventBuild(e.to_string()))?;
            events_out.push(ev);
        }
    }

    Ok((state, events_out))
}

/// Stage 5: finalise - accept an `EnrolmentDecision` and emit subject_finalised.
pub fn finalise(
    mut state: PipelineState,
    decision: &EnrolmentDecision,
) -> Result<(PipelineState, eel_events_rs::envelope::Envelope), EnrolmentError> {
    state.job.status = EnrolmentStatus::Enrolled;
    let ev = events::subject_finalised_event(
        decision.enrolment_job_id,
        decision.subject_id,
        &decision.gallery_shard_id,
    )
    .map_err(|e| EnrolmentError::EventBuild(e.to_string()))?;
    Ok((state, ev))
}
