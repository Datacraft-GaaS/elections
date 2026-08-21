//! Adjudication case FSM.
//!
//! Exhaustive state machine: every (state, action) pair is explicit.
//! Invalid transitions return `FsmError::InvalidTransition`.

use thiserror::Error;

use crate::domain::{AdjudicationDecision, CaseState, ReviewerTier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsmAction {
    Assign { reviewer_id: uuid::Uuid },
    StartReview,
    Decide { decision: AdjudicationDecision },
    EscalateTo { target_tier: ReviewerTier },
    AddEvidence,
    Withdraw { reason: String },
    Reopen,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FsmError {
    #[error("invalid transition: {from:?} + {action} -> not allowed")]
    InvalidTransition { from: CaseState, action: String },
    #[error("tier escalation requires supervisor approval")]
    EscalationRequiresSupervisor,
    #[error("panel vote requires 3-of-5 quorum; current approvals: {current}")]
    InsufficientPanelVotes { current: usize },
    #[error("reviewer has already voted")]
    AlreadyVoted,
}

/// Attempt a state transition, returning the new state on success.
///
/// Tier-specific invariants (escalation approval, panel quorum) are checked
/// here with the provided `context`. Pass `TransitionContext::default()` when
/// no extra invariants apply.
pub fn transition(
    from: &CaseState,
    action: &FsmAction,
    ctx: &TransitionContext,
) -> Result<CaseState, FsmError> {
    use CaseState::*;
    use FsmAction::*;

    match (from, action) {
        // Open: can only be assigned
        (Open, Assign { .. }) => Ok(Assigned),
        (Open, Withdraw { .. }) => Ok(Withdrawn),

        // Assigned: reviewer starts review or is reassigned
        (Assigned, StartReview) => Ok(InReview),
        (Assigned, Assign { .. }) => Ok(Assigned),
        (Assigned, Withdraw { .. }) => Ok(Withdrawn),

        // InReview: decide, escalate, or add evidence
        (InReview, Decide { .. }) => Ok(Decided),
        (InReview, AddEvidence) => Ok(InReview),
        (InReview, EscalateTo { target_tier }) => {
            // Escalation from Tier1 requires supervisor approval (ctx carries that flag).
            if !ctx.supervisor_approved && *target_tier == ReviewerTier::Tier2Supervisor {
                return Err(FsmError::EscalationRequiresSupervisor);
            }
            Ok(Escalated)
        }
        (InReview, Withdraw { .. }) => Ok(Withdrawn),

        // Escalated: re-assign at new tier
        (Escalated, Assign { .. }) => Ok(Assigned),
        (Escalated, Withdraw { .. }) => Ok(Withdrawn),

        // Withdrawn: can be reopened by supervisor
        (Withdrawn, Reopen) => {
            if !ctx.supervisor_approved {
                return Err(FsmError::EscalationRequiresSupervisor);
            }
            Ok(Reopened)
        }

        // Reopened: behaves like Open
        (Reopened, Assign { .. }) => Ok(Assigned),
        (Reopened, Withdraw { .. }) => Ok(Withdrawn),

        // Decided: terminal - no transitions allowed
        (Decided, _) => Err(FsmError::InvalidTransition {
            from: Decided,
            action: action_name(action),
        }),

        // Catch-all: invalid
        (state, act) => Err(FsmError::InvalidTransition {
            from: state.clone(),
            action: action_name(act),
        }),
    }
}

/// Determine whether a Tier3 panel has reached the required quorum.
///
/// Returns `Ok(true)` when >= 3 approvals exist out of the 5-member panel.
/// Returns `Err(FsmError::InsufficientPanelVotes)` otherwise.
pub fn panel_quorum_reached(approve_count: usize) -> Result<bool, FsmError> {
    if approve_count >= 3 {
        Ok(true)
    } else {
        Err(FsmError::InsufficientPanelVotes {
            current: approve_count,
        })
    }
}

/// Context passed to `transition()` for tier-specific invariant checks.
#[derive(Debug, Default)]
pub struct TransitionContext {
    /// Set to true when a supervisor has explicitly approved an escalation or reopen.
    pub supervisor_approved: bool,
}

fn action_name(action: &FsmAction) -> String {
    match action {
        FsmAction::Assign { .. } => "Assign".to_string(),
        FsmAction::StartReview => "StartReview".to_string(),
        FsmAction::Decide { .. } => "Decide".to_string(),
        FsmAction::EscalateTo { .. } => "EscalateTo".to_string(),
        FsmAction::AddEvidence => "AddEvidence".to_string(),
        FsmAction::Withdraw { .. } => "Withdraw".to_string(),
        FsmAction::Reopen => "Reopen".to_string(),
    }
}
