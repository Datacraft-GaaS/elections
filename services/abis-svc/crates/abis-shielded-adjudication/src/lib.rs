//! eel-abis-shielded-adjudication: encrypted-inputs adjudication for low-tier cases.
//!
//! Wraps eel-abis-shielding + eel-abis-cancelable + eel-abis-adjudication to
//! ensure probe and gallery templates never appear in plaintext for Tier-2 and
//! Tier-3 cases. Tier-1 election-critical cases retain plaintext dual-review.
//!
//! Key invariants:
//! - No raw biometric template bytes appear in reviewer-visible payloads or audit logs
//!   for shielded cases.
//! - The TierGate is the sole policy decision point for plaintext vs shielded.
//! - Every reviewer action is HSM-signed with a prev_action_hash chain.
//! - Shielded similarity scores match plaintext ABIS accuracy within eps=0.02.

pub mod audit;
pub mod case_shield;
pub mod reviewer_ui_hooks;
pub mod tier_gate;

pub use audit::{
    assert_no_plaintext_in_event, create_shielded_event, verify_shielded_chain, ShieldedAuditEvent,
};
pub use case_shield::{CaseShieldError, ShieldedCase};
pub use reviewer_ui_hooks::{
    assert_no_template_leak, build_payload, may_render_raw_image, ShieldedReviewPayload,
};
pub use tier_gate::{allows_plaintext, gate, CaseSeverity, ReviewMode};
