//! Integration and unit tests for eel-abis-adjudication.
//! All tests run without a live database; repository layer is excluded here.

use chrono::{Duration, Utc};
use uuid::Uuid;

use eel_abis_adjudication::{
    assignment::select_reviewer,
    audit::{create_action, verify_action_signature, verify_chain, AuditError},
    domain::{
        ActionType, AdjudicationCase, AdjudicationDecision, AdjudicationSlaPolicy,
        BiometricModality, CasePriority, CaseState, Reviewer, ReviewerTier,
    },
    events::{case_decided_event, case_opened_event, CASE_OPENED},
    fsm::{panel_quorum_reached, transition, FsmAction, FsmError, TransitionContext},
};

// ---- helpers ----

fn open_case() -> AdjudicationCase {
    AdjudicationCase::new(
        Some(Uuid::now_v7()),
        vec![Uuid::now_v7()],
        Uuid::now_v7(),
        CasePriority::Standard,
        ReviewerTier::Tier1Analyst,
    )
}

fn reviewer_pool() -> Vec<Reviewer> {
    vec![
        Reviewer {
            id: Uuid::now_v7(),
            tier: ReviewerTier::Tier1Analyst,
            specialisations: vec![BiometricModality::Fingerprint],
            active_case_count: 3,
        },
        Reviewer {
            id: Uuid::now_v7(),
            tier: ReviewerTier::Tier1Analyst,
            specialisations: vec![BiometricModality::Face, BiometricModality::Iris],
            active_case_count: 1,
        },
        Reviewer {
            id: Uuid::now_v7(),
            tier: ReviewerTier::Tier2Supervisor,
            specialisations: vec![BiometricModality::Fingerprint],
            active_case_count: 0,
        },
    ]
}

// ---- FSM tests ----

#[test]
fn open_case_starts_in_open_state() {
    let case = open_case();
    assert_eq!(case.state, CaseState::Open);
}

#[test]
fn assign_transitions_open_to_assigned() {
    let result = transition(
        &CaseState::Open,
        &FsmAction::Assign {
            reviewer_id: Uuid::now_v7(),
        },
        &TransitionContext::default(),
    );
    assert_eq!(result, Ok(CaseState::Assigned));
}

#[test]
fn start_review_transitions_assigned_to_in_review() {
    let result = transition(
        &CaseState::Assigned,
        &FsmAction::StartReview,
        &TransitionContext::default(),
    );
    assert_eq!(result, Ok(CaseState::InReview));
}

#[test]
fn tier1_confirm_match_transitions_to_decided() {
    let result = transition(
        &CaseState::InReview,
        &FsmAction::Decide {
            decision: AdjudicationDecision::ConfirmMatch(Uuid::now_v7()),
        },
        &TransitionContext::default(),
    );
    assert_eq!(result, Ok(CaseState::Decided));
}

#[test]
fn decided_is_terminal_no_further_transitions() {
    let result = transition(
        &CaseState::Decided,
        &FsmAction::Assign {
            reviewer_id: Uuid::now_v7(),
        },
        &TransitionContext::default(),
    );
    assert!(matches!(result, Err(FsmError::InvalidTransition { .. })));
}

#[test]
fn escalate_from_in_review_without_supervisor_approval_fails() {
    let result = transition(
        &CaseState::InReview,
        &FsmAction::EscalateTo {
            target_tier: ReviewerTier::Tier2Supervisor,
        },
        &TransitionContext {
            supervisor_approved: false,
        },
    );
    assert_eq!(result, Err(FsmError::EscalationRequiresSupervisor));
}

#[test]
fn escalate_from_in_review_with_supervisor_approval_succeeds() {
    let result = transition(
        &CaseState::InReview,
        &FsmAction::EscalateTo {
            target_tier: ReviewerTier::Tier2Supervisor,
        },
        &TransitionContext {
            supervisor_approved: true,
        },
    );
    assert_eq!(result, Ok(CaseState::Escalated));
}

#[test]
fn escalated_can_be_reassigned() {
    let result = transition(
        &CaseState::Escalated,
        &FsmAction::Assign {
            reviewer_id: Uuid::now_v7(),
        },
        &TransitionContext::default(),
    );
    assert_eq!(result, Ok(CaseState::Assigned));
}

#[test]
fn withdraw_from_open_is_valid() {
    let result = transition(
        &CaseState::Open,
        &FsmAction::Withdraw {
            reason: "duplicate submission".into(),
        },
        &TransitionContext::default(),
    );
    assert_eq!(result, Ok(CaseState::Withdrawn));
}

#[test]
fn reopen_withdrawn_requires_supervisor() {
    let result = transition(
        &CaseState::Withdrawn,
        &FsmAction::Reopen,
        &TransitionContext {
            supervisor_approved: false,
        },
    );
    assert_eq!(result, Err(FsmError::EscalationRequiresSupervisor));
}

#[test]
fn reopen_withdrawn_with_supervisor_succeeds() {
    let result = transition(
        &CaseState::Withdrawn,
        &FsmAction::Reopen,
        &TransitionContext {
            supervisor_approved: true,
        },
    );
    assert_eq!(result, Ok(CaseState::Reopened));
}

// ---- Panel quorum tests ----

#[test]
fn tier3_three_approvals_reaches_quorum() {
    assert!(panel_quorum_reached(3).is_ok());
}

#[test]
fn tier3_five_approvals_reaches_quorum() {
    assert!(panel_quorum_reached(5).is_ok());
}

#[test]
fn tier3_two_approvals_insufficient() {
    assert!(matches!(
        panel_quorum_reached(2),
        Err(FsmError::InsufficientPanelVotes { current: 2 })
    ));
}

#[test]
fn tier3_zero_approvals_insufficient() {
    assert!(matches!(
        panel_quorum_reached(0),
        Err(FsmError::InsufficientPanelVotes { current: 0 })
    ));
}

// ---- SLA tests ----

#[test]
fn tier1_sla_is_4_hours() {
    assert_eq!(AdjudicationSlaPolicy::TIER1_HOURS, 4);
}

#[test]
fn tier2_sla_is_12_hours() {
    assert_eq!(AdjudicationSlaPolicy::TIER2_HOURS, 12);
}

#[test]
fn tier3_sla_is_48_hours() {
    assert_eq!(AdjudicationSlaPolicy::TIER3_HOURS, 48);
}

#[test]
fn sla_breach_detected_when_deadline_in_past() {
    let past = Utc::now() - Duration::hours(1);
    assert!(AdjudicationSlaPolicy::is_breached(past));
}

#[test]
fn sla_not_breached_when_deadline_in_future() {
    let future = Utc::now() + Duration::hours(2);
    assert!(!AdjudicationSlaPolicy::is_breached(future));
}

// ---- Priority ordering tests ----

#[test]
fn election_day_hold_priority_is_highest() {
    assert!(CasePriority::ElectionDayHold > CasePriority::Urgent);
    assert!(CasePriority::Urgent > CasePriority::Standard);
}

// ---- Assignment tests ----

#[test]
fn reviewer_specialisation_routes_fingerprint_to_fingerprint_reviewer() {
    let pool = reviewer_pool();
    let selected = select_reviewer(
        &pool,
        &ReviewerTier::Tier1Analyst,
        &[BiometricModality::Fingerprint],
    );
    assert!(selected.is_some());
    let r = selected.unwrap();
    assert!(r.specialisations.contains(&BiometricModality::Fingerprint));
}

#[test]
fn reviewer_workload_balancing_picks_lowest_active_count() {
    let pool = reviewer_pool();
    // Both Tier1 analysts are in pool; fingerprint has 3 cases, face/iris has 1 case.
    // For Face modality, the face/iris analyst (count=1) should win.
    let selected = select_reviewer(
        &pool,
        &ReviewerTier::Tier1Analyst,
        &[BiometricModality::Face],
    );
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().active_case_count, 1);
}

#[test]
fn no_reviewer_available_for_missing_tier_returns_none() {
    let pool = reviewer_pool();
    let selected = select_reviewer(
        &pool,
        &ReviewerTier::Tier3Panel,
        &[BiometricModality::Fingerprint],
    );
    assert!(selected.is_none());
}

// ---- Audit chain tests ----

#[test]
fn first_action_has_no_prev_hash() {
    use eel_abis_adjudication::audit::test_utils::Ed25519TestSigner;
    let signer = Ed25519TestSigner::new();
    let action = create_action(
        Uuid::now_v7(),
        Uuid::now_v7(),
        ActionType::Assign,
        None,
        vec![],
        None,
        &signer,
    );
    assert!(action.prev_action_hash.is_none());
}

#[test]
fn second_action_chains_from_first() {
    use eel_abis_adjudication::audit::test_utils::Ed25519TestSigner;
    let signer = Ed25519TestSigner::new();
    let case_id = Uuid::now_v7();
    let reviewer = Uuid::now_v7();

    let a1 = create_action(
        case_id,
        reviewer,
        ActionType::Assign,
        None,
        vec![],
        None,
        &signer,
    );
    let a2 = create_action(
        case_id,
        reviewer,
        ActionType::StartReview,
        None,
        vec![],
        Some(&a1),
        &signer,
    );

    assert!(a2.prev_action_hash.is_some());
    assert_ne!(a2.prev_action_hash, a1.prev_action_hash);
}

#[test]
fn verify_chain_passes_for_valid_sequence() {
    use eel_abis_adjudication::audit::test_utils::Ed25519TestSigner;
    let signer = Ed25519TestSigner::new();
    let case_id = Uuid::now_v7();
    let reviewer = Uuid::now_v7();

    let a1 = create_action(
        case_id,
        reviewer,
        ActionType::Assign,
        None,
        vec![],
        None,
        &signer,
    );
    let a2 = create_action(
        case_id,
        reviewer,
        ActionType::StartReview,
        None,
        vec![],
        Some(&a1),
        &signer,
    );

    assert!(verify_chain(&[a1, a2]).is_ok());
}

#[test]
fn modified_action_breaks_hash_chain() {
    use eel_abis_adjudication::audit::test_utils::Ed25519TestSigner;
    let signer = Ed25519TestSigner::new();
    let case_id = Uuid::now_v7();
    let reviewer = Uuid::now_v7();

    let mut a1 = create_action(
        case_id,
        reviewer,
        ActionType::Assign,
        None,
        vec![],
        None,
        &signer,
    );
    let a2 = create_action(
        case_id,
        reviewer,
        ActionType::StartReview,
        None,
        vec![],
        Some(&a1),
        &signer,
    );

    // Tamper with a1's canonical JSON.
    a1.canonical_json = a1.canonical_json.replace("Assign", "Decide");

    // a2's prev_action_hash was computed before tampering, so chain is broken.
    let result = verify_chain(&[a1, a2]);
    assert!(matches!(result, Err(AuditError::ChainBroken { .. })));
}

#[test]
fn action_signature_verifies_correctly() {
    use eel_abis_adjudication::audit::test_utils::Ed25519TestSigner;
    let signer = Ed25519TestSigner::new();
    let action = create_action(
        Uuid::now_v7(),
        Uuid::now_v7(),
        ActionType::Decide,
        Some("confirmed duplicate".into()),
        vec![],
        None,
        &signer,
    );
    assert!(verify_action_signature(&action).is_ok());
}

// ---- Event envelope tests ----

#[test]
fn case_opened_event_has_correct_type() {
    let case = open_case();
    let env = case_opened_event(&case).unwrap();
    assert_eq!(env.event_type, CASE_OPENED);
}

#[test]
fn case_decided_event_contains_case_id() {
    let case_id = Uuid::now_v7();
    let env = case_decided_event(case_id, &AdjudicationDecision::ConfirmNoMatch).unwrap();
    assert!(env.data["case_id"]
        .as_str()
        .unwrap()
        .contains(&case_id.to_string()[..8]));
}
