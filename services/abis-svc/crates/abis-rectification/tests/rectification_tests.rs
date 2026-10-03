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

//! Tests for rectification workflow.

use eel_abis_rectification::domain::{
    RectificationRequest, RectificationRequestType, RectificationState, SlaStatus,
};
use eel_abis_rectification::error::RectificationError;
use eel_abis_rectification::policy::PolicyEngine;
use eel_abis_rectification::scheduler::sweep_sla;
use eel_abis_rectification::workflow::{
    assign_officer, complete, decide, to_evidence_pending, to_triage,
};
use uuid::Uuid;

fn new_req(t: RectificationRequestType) -> RectificationRequest {
    RectificationRequest::new(Uuid::now_v7(), t)
}

#[test]
fn new_request_is_submitted() {
    let r = new_req(RectificationRequestType::DemographicUpdate);
    assert_eq!(r.state, RectificationState::Submitted);
    assert_eq!(r.sla_status(), SlaStatus::OnTrack);
}

#[test]
fn happy_path_biometric_recapture() {
    let mut r = new_req(RectificationRequestType::BiometricRecapture);
    let officer = Uuid::now_v7();
    to_triage(&mut r).unwrap();
    assign_officer(&mut r, officer).unwrap();
    decide(&mut r, true, "approved", officer).unwrap();
    complete(&mut r).unwrap();
    assert_eq!(r.state, RectificationState::Completed);
    assert!(r.completed_at.is_some());
}

#[test]
fn triage_from_wrong_state_fails() {
    let mut r = new_req(RectificationRequestType::DemographicUpdate);
    to_triage(&mut r).unwrap();
    assert!(matches!(
        to_triage(&mut r),
        Err(RectificationError::InvalidTransition { .. })
    ));
}

#[test]
fn template_removal_requires_odpc_ref() {
    let mut r = new_req(RectificationRequestType::TemplateRemoval);
    let officer = Uuid::now_v7();
    r.assigned_officer = Some(officer);
    to_triage(&mut r).unwrap();
    assert!(matches!(
        decide(&mut r, true, "ok", officer),
        Err(RectificationError::OdpcRefRequired)
    ));
}

#[test]
fn template_removal_approved_with_odpc_ref() {
    let mut r = new_req(RectificationRequestType::TemplateRemoval);
    let officer = Uuid::now_v7();
    r.assigned_officer = Some(officer);
    r.odpc_ref = Some("ODPC-2026-001".to_string());
    to_triage(&mut r).unwrap();
    decide(&mut r, true, "ok", officer).unwrap();
    assert_eq!(r.state, RectificationState::Approved);
}

#[test]
fn rejection_sets_state_rejected() {
    let mut r = new_req(RectificationRequestType::DemographicUpdate);
    let officer = Uuid::now_v7();
    to_triage(&mut r).unwrap();
    decide(&mut r, false, "insufficient evidence", officer).unwrap();
    assert_eq!(r.state, RectificationState::Rejected);
}

#[test]
fn complete_requires_approved_state() {
    let mut r = new_req(RectificationRequestType::DemographicUpdate);
    to_triage(&mut r).unwrap();
    assert!(matches!(
        complete(&mut r),
        Err(RectificationError::InvalidTransition { .. })
    ));
}

#[test]
fn sla_on_track_initially() {
    let r = new_req(RectificationRequestType::BiometricRecapture);
    assert_eq!(r.sla_status(), SlaStatus::OnTrack);
}

#[test]
fn sla_sweep_all_on_track() {
    let requests: Vec<_> = (0..3)
        .map(|_| new_req(RectificationRequestType::DemographicUpdate))
        .collect();
    let (near, breached) = sweep_sla(&requests);
    assert_eq!(near, 0);
    assert_eq!(breached, 0);
}

#[test]
fn policy_requires_officer_for_biometric_recapture() {
    let policy = PolicyEngine;
    assert!(policy.requires_officer(&RectificationRequestType::BiometricRecapture));
    assert!(!policy.requires_officer(&RectificationRequestType::DemographicUpdate));
}

#[test]
fn policy_odpc_required_for_template_removal() {
    let policy = PolicyEngine;
    let evidence = policy.required_evidence(&RectificationRequestType::TemplateRemoval);
    assert!(matches!(
        evidence,
        Some(eel_abis_rectification::domain::EvidenceType::OdpcReference)
    ));
}

#[test]
fn evidence_pending_transition() {
    let mut r = new_req(RectificationRequestType::DemographicUpdate);
    to_triage(&mut r).unwrap();
    to_evidence_pending(&mut r).unwrap();
    assert_eq!(r.state, RectificationState::EvidencePending);
}

#[test]
fn assign_officer_in_triage() {
    let mut r = new_req(RectificationRequestType::BiometricRecapture);
    let officer = Uuid::now_v7();
    to_triage(&mut r).unwrap();
    assign_officer(&mut r, officer).unwrap();
    assert_eq!(r.assigned_officer, Some(officer));
}

#[test]
fn new_request_sla_deadline_is_30_days() {
    let r = new_req(RectificationRequestType::DuplicateMerge);
    let diff = r
        .sla_deadline
        .signed_duration_since(r.submitted_at)
        .num_days();
    assert_eq!(diff, 30);
}
