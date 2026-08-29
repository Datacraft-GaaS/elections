//! `eel-abis-adjudication`: adjudication workflow, FSM, reviewer audit chain.
//!
//! Expanded from the stub in abis-matcher/src/adjudication.rs into a full crate.
//! The abis-matcher stub is left in place; this crate is the real implementation.
#![warn(missing_docs)]

pub mod assignment;
pub mod audit;
pub mod domain;
pub mod error;
pub mod events;
pub mod fsm;
pub mod repository;

pub use domain::{
    AdjudicationCase, AdjudicationDecision, AdjudicationSlaPolicy, ActionType, BiometricModality,
    CasePriority, CaseState, PanelVote, Reviewer, ReviewerAction, ReviewerTier, Vote,
};
pub use error::AdjudicationError;
pub use fsm::{panel_quorum_reached, transition, FsmAction, FsmError, TransitionContext};
