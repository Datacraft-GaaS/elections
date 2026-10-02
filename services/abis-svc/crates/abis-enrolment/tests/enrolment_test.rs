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

//! Unit tests for the enrolment pipeline.
//! All tests run without a live database or matcher service.

use chrono::Utc;
use uuid::Uuid;

use eel_abis_enrolment::{
    domain::{
        CapturedTemplate, DedupDecision, DedupResult, EnrolmentDecision, EnrolmentJob,
        EnrolmentSource, EnrolmentStatus, FingerPosition, Modality, PadResult, ProbableMatch,
        QualityReport,
    },
    events::{DEDUP_HOLD, ENROLLED, ENROLMENT_RECEIVED, ENROLMENT_REJECTED, SUBJECT_FINALISED},
    pipeline::{apply_dedup_result, apply_quality_report, finalise, ingest, route_decision},
};

// ---- helpers ----

fn make_job() -> EnrolmentJob {
    EnrolmentJob::new(
        Uuid::now_v7(),
        EnrolmentSource::BvrKit,
        Utc::now(),
        Some("kit-001".into()),
        Some("officer-007".into()),
        sample_templates(),
    )
}

fn sample_templates() -> Vec<CapturedTemplate> {
    (0..8)
        .map(|i| CapturedTemplate {
            modality: Modality::Fingerprint,
            finger_position: Some(FingerPosition::RightIndex),
            quality_score: 0.85,
            template_bytes_ref: format!("blake3:abc{i}"),
            pad_result: PadResult::Live,
        })
        .collect()
}

fn good_quality_report() -> QualityReport {
    QualityReport {
        per_modality: vec![(Modality::Fingerprint, 0.85), (Modality::Face, 0.90)],
        usable_finger_count: 8,
        iris_captured: false,
        face_captured: true,
        retake_reasons: vec![],
    }
}

fn bad_quality_report() -> QualityReport {
    QualityReport {
        per_modality: vec![(Modality::Fingerprint, 0.40)],
        usable_finger_count: 3,
        iris_captured: false,
        face_captured: false,
        retake_reasons: vec!["face_missing".into(), "insufficient_fingers".into()],
    }
}

fn dedup_no_match() -> DedupResult {
    DedupResult {
        probable_matches: vec![],
        decision: DedupDecision::NoMatch,
    }
}

fn dedup_high_confidence() -> DedupResult {
    DedupResult {
        probable_matches: vec![ProbableMatch {
            subject_id: Uuid::now_v7(),
            score: 0.97,
            modality_contributions: vec![(Modality::Fingerprint, 0.97)],
        }],
        decision: DedupDecision::HighConfidenceMatch,
    }
}

fn dedup_manual() -> DedupResult {
    DedupResult {
        probable_matches: vec![ProbableMatch {
            subject_id: Uuid::now_v7(),
            score: 0.75,
            modality_contributions: vec![(Modality::Face, 0.75)],
        }],
        decision: DedupDecision::ManualAdjudicationRequired,
    }
}

// ---- Stage 1: ingest ----

#[test]
fn ingest_creates_received_job_and_event() {
    let job = make_job();
    let (state, env) = ingest(job).unwrap();
    assert_eq!(state.job.status, EnrolmentStatus::Received);
    assert_eq!(env.event_type, ENROLMENT_RECEIVED);
}

#[test]
fn ingested_job_has_uuid7_id() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    // UUID v7 has version nibble = 7
    assert_eq!(state.job.id.get_version_num(), 7);
}

#[test]
fn ingest_event_contains_subject_id() {
    let job = make_job();
    let subject_id = job.subject_id;
    let (_, env) = ingest(job).unwrap();
    assert_eq!(
        env.data["subject_id"].as_str().unwrap(),
        subject_id.to_string()
    );
}

// ---- Stage 2: quality check ----

#[test]
fn quality_pass_advances_to_dedup_pending() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, events) = apply_quality_report(state, good_quality_report()).unwrap();
    assert_eq!(state.job.status, EnrolmentStatus::DedupPending);
    assert!(events.is_empty(), "no events emitted on quality pass");
}

#[test]
fn quality_fail_sets_rejected_and_emits_event() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, events) = apply_quality_report(state, bad_quality_report()).unwrap();
    assert_eq!(state.job.status, EnrolmentStatus::Rejected);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, ENROLMENT_REJECTED);
}

#[test]
fn quality_fail_event_contains_reasons() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (_, events) = apply_quality_report(state, bad_quality_report()).unwrap();
    let reasons = events[0].data["reasons"].as_array().unwrap();
    assert!(!reasons.is_empty());
}

#[test]
fn quality_report_stored_on_job() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    assert!(state.job.quality_report.is_some());
}

// ---- Stage 3: dedup ----

#[test]
fn dedup_result_stored_and_status_advances() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    let state = apply_dedup_result(state, dedup_no_match());
    assert_eq!(state.job.status, EnrolmentStatus::DedupComplete);
    assert!(state.dedup_result.is_some());
}

#[test]
fn dedup_result_preserves_modality_contributions() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    let state = apply_dedup_result(state, dedup_high_confidence());
    let result = state.dedup_result.as_ref().unwrap();
    assert!(!result.probable_matches.is_empty());
    assert!(!result.probable_matches[0].modality_contributions.is_empty());
}

// ---- Stage 4: route_decision ----

#[test]
fn no_match_routes_to_enrolled() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    let state = apply_dedup_result(state, dedup_no_match());
    let (state, events) = route_decision(state).unwrap();
    assert_eq!(state.job.status, EnrolmentStatus::Enrolled);
    assert_eq!(events[0].event_type, ENROLLED);
}

#[test]
fn enrolled_event_contains_subject_id() {
    let job = make_job();
    let subject_id = job.subject_id;
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    let state = apply_dedup_result(state, dedup_no_match());
    let (_, events) = route_decision(state).unwrap();
    assert_eq!(
        events[0].data["subject_id"].as_str().unwrap(),
        subject_id.to_string()
    );
}

#[test]
fn high_confidence_match_routes_to_dedup_hold() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    let state = apply_dedup_result(state, dedup_high_confidence());
    let (state, events) = route_decision(state).unwrap();
    assert_eq!(state.job.status, EnrolmentStatus::DedupHold);
    assert_eq!(events[0].event_type, DEDUP_HOLD);
}

#[test]
fn manual_adjudication_routes_to_adjudication_required() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    let state = apply_dedup_result(state, dedup_manual());
    let (state, events) = route_decision(state).unwrap();
    assert_eq!(state.job.status, EnrolmentStatus::AdjudicationRequired);
    assert_eq!(events[0].event_type, DEDUP_HOLD);
}

#[test]
fn route_decision_without_dedup_result_errors() {
    let job = make_job();
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    // Skipped apply_dedup_result intentionally.
    let result = route_decision(state);
    assert!(result.is_err());
}

// ---- Stage 5: finalise ----

#[test]
fn finalise_emits_subject_finalised_event() {
    let job = make_job();
    let job_id = job.id;
    let subject_id = job.subject_id;
    let (state, _) = ingest(job).unwrap();
    let (state, _) = apply_quality_report(state, good_quality_report()).unwrap();
    let state = apply_dedup_result(state, dedup_no_match());
    let (state, _) = route_decision(state).unwrap();

    let decision = EnrolmentDecision {
        id: Uuid::now_v7(),
        enrolment_job_id: job_id,
        subject_id,
        enrolled_at: Utc::now(),
        gallery_shard_id: "shard-7".into(),
        previous_records: vec![],
        merge_from: None,
        adjudicator_id: None,
    };

    let (final_state, ev) = finalise(state, &decision).unwrap();
    assert_eq!(final_state.job.status, EnrolmentStatus::Enrolled);
    assert_eq!(ev.event_type, SUBJECT_FINALISED);
}

// ---- Idempotency ----

#[test]
fn two_ingest_calls_produce_different_job_ids() {
    let job1 = make_job();
    let job2 = make_job();
    // Different jobs must have different UUIDs (v7 monotonic).
    assert_ne!(job1.id, job2.id);
}

// ---- Event envelope structure ----

#[test]
fn enrolment_received_event_has_spec_version_1_0() {
    let job = make_job();
    let (_, env) = ingest(job).unwrap();
    assert_eq!(env.specversion, "1.0");
}

#[test]
fn all_events_have_eel_stream_id() {
    let job = make_job();
    let (_state, env) = ingest(job).unwrap();
    // stream_id is non-empty
    assert!(!env.eel_stream_id.is_empty());
}
