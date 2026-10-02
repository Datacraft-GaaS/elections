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

//! Shielded-review audit: every reviewer action on a shielded case is HSM-signed
//! with a prev_action_hash chain so the review history is tamper-evident.
//!
//! This module wraps the core adjudication audit (eel-abis-adjudication) and
//! adds shielded-specific event emission: no plaintext template ever appears in
//! the emitted audit log or in the canonical JSON that is signed.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use eel_abis_adjudication::{
    audit::{create_action, verify_chain, AuditError, HsmSigner},
    ActionType, ReviewerAction,
};

use crate::tier_gate::ReviewMode;

/// A shielded audit event emitted when a reviewer takes an action on a shielded case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedAuditEvent {
    /// The underlying signed reviewer action.
    pub action: ReviewerAction,
    /// The review mode under which this action was taken.
    pub review_mode: ReviewMode,
    /// Similarity score that was visible to the reviewer (shielded, not raw).
    pub visible_similarity: f64,
    /// Popcount of probe Bloom filter (no raw template data).
    pub probe_popcount: usize,
}

/// Create a shielded audit event, chaining from the previous action.
///
/// The `visible_similarity` and `probe_popcount` are shielded-layer metrics;
/// no plaintext template is included in the event.
pub fn create_shielded_event(
    case_id: Uuid,
    reviewer_id: Uuid,
    action_type: ActionType,
    comment: Option<String>,
    prev_action: Option<&ReviewerAction>,
    review_mode: ReviewMode,
    visible_similarity: f64,
    probe_popcount: usize,
    signer: &dyn HsmSigner,
) -> ShieldedAuditEvent {
    let action = create_action(
        case_id,
        reviewer_id,
        action_type,
        comment,
        vec![],
        prev_action,
        signer,
    );
    ShieldedAuditEvent {
        action,
        review_mode,
        visible_similarity,
        probe_popcount,
    }
}

/// Verify that a sequence of shielded audit events forms an unbroken chain.
///
/// Extracts the `ReviewerAction` layer and delegates to the core chain verifier.
pub fn verify_shielded_chain(events: &[ShieldedAuditEvent]) -> Result<(), AuditError> {
    let actions: Vec<&ReviewerAction> = events.iter().map(|e| &e.action).collect();
    let owned: Vec<ReviewerAction> = actions.iter().map(|a| (*a).clone()).collect();
    verify_chain(&owned)
}

/// Assert that a shielded audit event contains no raw template bytes.
///
/// Checks the canonical_json of the action for any occurrence of `probe_hex`.
pub fn assert_no_plaintext_in_event(
    event: &ShieldedAuditEvent,
    probe_hex: &str,
) -> Result<(), String> {
    if event.action.canonical_json.contains(probe_hex) {
        return Err(format!(
            "plaintext template found in audit event for case {}",
            event.action.case_id
        ));
    }
    let serialised = serde_json::to_string(event).unwrap_or_default();
    if serialised.contains(probe_hex) {
        return Err(format!(
            "plaintext template found in serialised audit event for case {}",
            event.action.case_id
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_adjudication::audit::test_utils::Ed25519TestSigner;
    use eel_abis_adjudication::ActionType;

    fn signer() -> Ed25519TestSigner {
        Ed25519TestSigner::new()
    }

    fn case_id() -> Uuid {
        Uuid::now_v7()
    }

    fn reviewer_id() -> Uuid {
        Uuid::now_v7()
    }

    #[test]
    fn creates_shielded_event() {
        let s = signer();
        let event = create_shielded_event(
            case_id(),
            reviewer_id(),
            ActionType::Assign,
            None,
            None,
            ReviewMode::ShieldedReview,
            0.72,
            128,
            &s,
        );
        assert_eq!(event.visible_similarity, 0.72);
        assert_eq!(event.probe_popcount, 128);
        assert_eq!(event.review_mode, ReviewMode::ShieldedReview);
    }

    #[test]
    fn chain_of_two_events_valid() {
        let s = signer();
        let cid = case_id();
        let rid = reviewer_id();
        let e1 = create_shielded_event(
            cid,
            rid,
            ActionType::Assign,
            None,
            None,
            ReviewMode::ShieldedReview,
            0.5,
            100,
            &s,
        );
        let e2 = create_shielded_event(
            cid,
            rid,
            ActionType::Decide,
            Some("shielded match confirmed".to_string()),
            Some(&e1.action),
            ReviewMode::ShieldedReview,
            0.5,
            100,
            &s,
        );
        let events = [e1, e2];
        verify_shielded_chain(&events).unwrap();
    }

    #[test]
    fn broken_chain_detected() {
        let s = signer();
        let cid = case_id();
        let rid = reviewer_id();
        let e1 = create_shielded_event(
            cid,
            rid,
            ActionType::Assign,
            None,
            None,
            ReviewMode::ShieldedReview,
            0.5,
            100,
            &s,
        );
        // Create e2 without linking to e1 (prev_action = None).
        let e2 = create_shielded_event(
            cid,
            rid,
            ActionType::Decide,
            None,
            None, // broken: should be Some(&e1.action)
            ReviewMode::ShieldedReview,
            0.5,
            100,
            &s,
        );
        let events = [e1, e2];
        assert!(verify_shielded_chain(&events).is_err());
    }

    #[test]
    fn no_plaintext_in_event() {
        let s = signer();
        let event = create_shielded_event(
            case_id(),
            reviewer_id(),
            ActionType::Assign,
            None,
            None,
            ReviewMode::ShieldedReview,
            0.8,
            200,
            &s,
        );
        let fake_probe_hex = hex::encode(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(assert_no_plaintext_in_event(&event, &fake_probe_hex).is_ok());
    }
}
