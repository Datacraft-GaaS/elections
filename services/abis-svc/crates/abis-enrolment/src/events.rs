//! CloudEvent envelope builders for enrolment lifecycle events.

use uuid::Uuid;

use crate::domain::{DedupDecision, EnrolmentJob, QualityReport};

pub const ENROLMENT_RECEIVED: &str = "eel.abis.enrolment_received.v1";
pub const ENROLMENT_REJECTED: &str = "eel.abis.enrolment_rejected.v1";
pub const ENROLLED: &str = "eel.abis.enrolled.v1";
pub const DEDUP_HOLD: &str = "eel.abis.dedup_hold.v1";
pub const SUBJECT_FINALISED: &str = "eel.abis.subject_finalised.v1";
pub const QUALITY_PASSED: &str = "eel.abis.quality_passed.v1";

const SOURCE: &str = "spiffe://eel.local/service/abis-enrolment";

type BuildResult = Result<eel_events_rs::envelope::Envelope, eel_events_rs::error::EventError>;

pub fn enrolment_received_event(job: &EnrolmentJob) -> BuildResult {
    eel_events_rs::EventBuilder::new()
        .event_type(ENROLMENT_RECEIVED)
        .source(SOURCE)
        .stream_id(job.id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(job.subject_id.to_string())
        .data(serde_json::json!({
            "job_id": job.id.to_string(),
            "subject_id": job.subject_id.to_string(),
            "source": job.source.as_str(),
            "captured_at": job.captured_at.to_rfc3339(),
        }))
        .data_class(eel_events_rs::envelope::DataClass::A)
        .build_unsigned()
}

pub fn enrolment_rejected_event(job: &EnrolmentJob, reasons: &[String]) -> BuildResult {
    eel_events_rs::EventBuilder::new()
        .event_type(ENROLMENT_REJECTED)
        .source(SOURCE)
        .stream_id(job.id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(job.subject_id.to_string())
        .data(serde_json::json!({
            "job_id": job.id.to_string(),
            "subject_id": job.subject_id.to_string(),
            "reasons": reasons,
        }))
        .data_class(eel_events_rs::envelope::DataClass::A)
        .build_unsigned()
}

pub fn enrolled_event(job_id: Uuid, subject_id: Uuid, gallery_shard_id: &str) -> BuildResult {
    eel_events_rs::EventBuilder::new()
        .event_type(ENROLLED)
        .source(SOURCE)
        .stream_id(job_id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(subject_id.to_string())
        .data(serde_json::json!({
            "job_id": job_id.to_string(),
            "subject_id": subject_id.to_string(),
            "gallery_shard_id": gallery_shard_id,
        }))
        .data_class(eel_events_rs::envelope::DataClass::A)
        .build_unsigned()
}

pub fn dedup_hold_event(job_id: Uuid, subject_id: Uuid, decision: &DedupDecision) -> BuildResult {
    eel_events_rs::EventBuilder::new()
        .event_type(DEDUP_HOLD)
        .source(SOURCE)
        .stream_id(job_id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(subject_id.to_string())
        .data(serde_json::json!({
            "job_id": job_id.to_string(),
            "subject_id": subject_id.to_string(),
            "decision": format!("{:?}", decision),
        }))
        .data_class(eel_events_rs::envelope::DataClass::A)
        .build_unsigned()
}

pub fn subject_finalised_event(
    job_id: Uuid,
    subject_id: Uuid,
    gallery_shard_id: &str,
) -> BuildResult {
    eel_events_rs::EventBuilder::new()
        .event_type(SUBJECT_FINALISED)
        .source(SOURCE)
        .stream_id(job_id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(subject_id.to_string())
        .data(serde_json::json!({
            "job_id": job_id.to_string(),
            "subject_id": subject_id.to_string(),
            "gallery_shard_id": gallery_shard_id,
        }))
        .data_class(eel_events_rs::envelope::DataClass::A)
        .build_unsigned()
}
