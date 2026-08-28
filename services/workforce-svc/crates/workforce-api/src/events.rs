//! Cross-service events emitted by workforce-svc (design §8).
//!
//! Pure [`OutboxEnvelope`] builders: the topic/payload contracts here are what
//! downstream consumers (payroll M55, training M38, field-mobile, inventory,
//! ecc, transparency) depend on. Payloads carry ids and hashes only — never
//! raw PII (DPA 2019). Emission happens transactionally in [`crate::store`].

use chrono::Utc;
use eel_outbox_rs::OutboxEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::domain::{
    EquipmentClaim, PerformanceRecord, PostingOrder, StageChange, TaskAssignment, Worker,
};

/// SPIFFE identity of this service, used as the event source.
pub const SOURCE_URI: &str = "spiffe://eel.local/service/workforce";

pub mod topics {
    pub const APPLICATION_STAGE_CHANGED: &str = "eel.workforce.application.stage_changed.v1";
    pub const WORKER_APPOINTED: &str = "eel.workforce.worker.appointed.v1";
    pub const POSTING_ACTIVATED: &str = "eel.workforce.posting.activated.v1";
    pub const KIT_ISSUED: &str = "eel.workforce.equipment.kit_issued.v1";
    pub const KIT_RETURNED: &str = "eel.workforce.equipment.kit_returned.v1";
    pub const TASK_COMPLETED: &str = "eel.workforce.task.completed.v1";
    pub const PERFORMANCE_RECORDED: &str = "eel.workforce.performance.evaluation_recorded.v1";
}

fn envelope(
    topic: &str,
    stream_id: Uuid,
    subject: Option<&str>,
    data: serde_json::Value,
    aggregate_type: &str,
    aggregate_id: &str,
) -> OutboxEnvelope {
    let now = Utc::now();
    OutboxEnvelope {
        id: Uuid::now_v7().to_string(),
        event_type: topic.to_string(),
        source: SOURCE_URI.to_string(),
        subject: subject.map(str::to_string),
        time: now.to_rfc3339(),
        eel_stream_id: stream_id.to_string(),
        eel_seq: 0,
        eel_hlc: now.timestamp_millis(),
        eel_prev_hash: None,
        eel_correlation_id: None,
        eel_causation_id: None,
        eel_producer_key_id: None,
        eel_signature: None,
        eel_election_id: None,
        eel_data_class: Some("B".to_string()),
        data,
        topic: topic.to_string(),
        kafka_key: stream_id.to_string(),
        aggregate_type: aggregate_type.to_string(),
        aggregate_id: aggregate_id.to_string(),
    }
}

/// Recruitment stage transition (any pipeline edge, including rejection and
/// waitlisting). Consumed by analytics and downstream gate owners.
pub fn stage_changed_envelope(case_id: Uuid, change: &StageChange) -> OutboxEnvelope {
    envelope(
        topics::APPLICATION_STAGE_CHANGED,
        case_id,
        Some(&format!("case/{case_id}")),
        json!({
            "case_id": case_id,
            "from": change.from,
            "to": change.to,
            "actor": change.actor,
            "reason": change.reason,
        }),
        "recruitment_case",
        &case_id.to_string(),
    )
}

/// A candidate was appointed and a worker record created. Consumed by payroll
/// (M55), training (M38) and field-mobile for onboarding.
pub fn worker_appointed_envelope(worker: &Worker, contract_ref: &str) -> OutboxEnvelope {
    envelope(
        topics::WORKER_APPOINTED,
        worker.worker_id,
        Some(&format!("worker/{}", worker.worker_id)),
        json!({
            "worker_id": worker.worker_id,
            "national_id_hash": worker.national_id_hash,
            "home_county": worker.home_county,
            "role_eligibility": worker.role_eligibility,
            "contract_ref": contract_ref,
        }),
        "worker",
        &worker.worker_id.to_string(),
    )
}

/// A posting was activated for a worker. Consumed by field-mobile to notify the
/// worker where they are posted.
pub fn posting_activated_envelope(posting: &PostingOrder) -> OutboxEnvelope {
    envelope(
        topics::POSTING_ACTIVATED,
        posting.posting_id,
        Some(&format!("posting/{}", posting.posting_id)),
        json!({
            "posting_id": posting.posting_id,
            "worker_id": posting.worker_id,
            "station_code": posting.station_code,
            "role_code": posting.role_code,
            "effective_from": posting.effective_from,
        }),
        "posting",
        &posting.posting_id.to_string(),
    )
}

/// Equipment was issued against a posting (custody chain). Consumed by
/// inventory-svc to record a `DISPATCHED` custody transition.
pub fn kit_issued_envelope(claim: &EquipmentClaim) -> OutboxEnvelope {
    envelope(
        topics::KIT_ISSUED,
        claim.claim_id,
        Some(&format!("claim/{}", claim.claim_id)),
        json!({
            "claim_id": claim.claim_id,
            "posting_id": claim.posting_id,
            "kit_serials": claim.kit_serials,
        }),
        "equipment_claim",
        &claim.claim_id.to_string(),
    )
}

/// Equipment was returned, closing the custody loop. Consumed by inventory-svc
/// to record a `SEALED_RETURN` transition.
pub fn kit_returned_envelope(claim: &EquipmentClaim) -> OutboxEnvelope {
    envelope(
        topics::KIT_RETURNED,
        claim.claim_id,
        Some(&format!("claim/{}", claim.claim_id)),
        json!({
            "claim_id": claim.claim_id,
            "posting_id": claim.posting_id,
            "loss_flags": claim.loss_flags,
        }),
        "equipment_claim",
        &claim.claim_id.to_string(),
    )
}

/// A task was completed. The critical subset is anchored downstream to the
/// transparency log; all completions feed the ECC dashboard.
pub fn task_completed_envelope(assignment: &TaskAssignment) -> OutboxEnvelope {
    envelope(
        topics::TASK_COMPLETED,
        assignment.assignment_id,
        Some(&format!("assignment/{}", assignment.assignment_id)),
        json!({
            "assignment_id": assignment.assignment_id,
            "posting_id": assignment.posting_id,
            "template_id": assignment.template_id,
            "evidence_hash": assignment.evidence_hash,
        }),
        "task_assignment",
        &assignment.assignment_id.to_string(),
    )
}

/// A performance record was appended. Consumed by payroll (M55) for breach
/// referral and by analytics for pool tiering.
pub fn performance_recorded_envelope(record: &PerformanceRecord) -> OutboxEnvelope {
    envelope(
        topics::PERFORMANCE_RECORDED,
        record.record_id,
        Some(&format!("worker/{}/performance", record.worker_id)),
        json!({
            "record_id": record.record_id,
            "worker_id": record.worker_id,
            "category": record.category.as_str(),
            "score": record.score,
            "recorded_by": record.recorded_by,
        }),
        "performance_record",
        &record.record_id.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{PerfCategory, TaskStatus, Worker};

    fn worker() -> Worker {
        Worker {
            worker_id: Uuid::now_v7(),
            legal_name: "Ada Oburu".into(),
            national_id_hash: "a".repeat(64),
            home_county: "KE-030".into(),
            role_eligibility: vec!["PO".into()],
            payroll_contract_ref: None,
            next_of_kin_name: None,
            next_of_kin_phone_hash: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn worker_appointed_carries_ids_and_hashes_not_pii() {
        let w = worker();
        let env = worker_appointed_envelope(&w, "SC-1");

        assert_eq!(env.topic, topics::WORKER_APPOINTED);
        assert_eq!(env.aggregate_type, "worker");
        assert_eq!(env.aggregate_id, w.worker_id.to_string());
        assert_eq!(env.eel_data_class.as_deref(), Some("B"));
        assert_eq!(env.data["national_id_hash"], w.national_id_hash);
        // Legal name (PII) must never leave the service.
        assert!(env.data.get("legal_name").is_none());
    }

    #[test]
    fn stage_changed_carries_transition_attribution() {
        let case_id = Uuid::now_v7();
        let change = StageChange {
            from: "screened".into(),
            to: "documents_complete".into(),
            actor: "compliance-1".into(),
            at: Utc::now(),
            reason: None,
        };
        let env = stage_changed_envelope(case_id, &change);

        assert_eq!(env.topic, topics::APPLICATION_STAGE_CHANGED);
        assert_eq!(env.data["from"], "screened");
        assert_eq!(env.data["to"], "documents_complete");
        assert_eq!(env.data["actor"], "compliance-1");
    }

    #[test]
    fn every_topic_prefixes_eel_workforce() {
        let w = worker();
        let case_id = Uuid::now_v7();
        let posting = PostingOrder {
            posting_id: Uuid::now_v7(),
            worker_id: w.worker_id,
            station_code: "STN-1".into(),
            role_code: "PO".into(),
            effective_from: Utc::now(),
            effective_to: None,
            posted_by: "cro".into(),
            superseded_by: None,
            status: "active".into(),
            created_at: Utc::now(),
        };
        let claim = EquipmentClaim {
            claim_id: Uuid::now_v7(),
            posting_id: posting.posting_id,
            kit_serials: vec!["BVI-1".into()],
            acknowledged: false,
            returned: false,
            loss_flags: vec![],
            created_at: Utc::now(),
        };
        let assignment = TaskAssignment {
            assignment_id: Uuid::now_v7(),
            posting_id: posting.posting_id,
            template_id: Uuid::now_v7(),
            due_from: Utc::now(),
            due_to: Utc::now(),
            status: TaskStatus::Done,
            evidence_hash: Some("b".repeat(64)),
            block_reason: None,
            updated_at: Utc::now(),
        };
        let record = PerformanceRecord {
            record_id: Uuid::now_v7(),
            worker_id: w.worker_id,
            category: PerfCategory::Attendance,
            score: Some(4.0),
            detail: "early".into(),
            recorded_by: "ro".into(),
            recorded_at: Utc::now(),
        };

        let envs = vec![
            stage_changed_envelope(
                case_id,
                &StageChange {
                    from: "applied".into(),
                    to: "screened".into(),
                    actor: "a".into(),
                    at: Utc::now(),
                    reason: None,
                },
            ),
            worker_appointed_envelope(&w, "SC-1"),
            posting_activated_envelope(&posting),
            kit_issued_envelope(&claim),
            kit_returned_envelope(&claim),
            task_completed_envelope(&assignment),
            performance_recorded_envelope(&record),
        ];
        for e in envs {
            assert!(e.topic.starts_with("eel.workforce."), "topic {}", e.topic);
        }
    }
}
