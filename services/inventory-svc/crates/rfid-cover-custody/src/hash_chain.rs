//! BLAKE3 hash chain over `eel_cover_state_transitions`.
//!
//! Each transition hashes `prev_hash || canonical_json(transition_data)`.
//! A silent rewrite breaks the chain; `verify_chain` reconstructs and rejects.

use crate::domain::CoverStateTransition;
use crate::error::RfidError;

/// Compute `BLAKE3(prev_hash || canonical_json(transition))`.
///
/// `prev_hash` is `&[]` for the genesis (first) row of a tag.
pub fn compute_this_hash(prev_hash: &[u8], transition: &CoverStateTransition) -> [u8; 32] {
    let canonical = serde_json::json!({
        "transition_id": transition.transition_id.to_string(),
        "tag_id":         transition.tag_id.to_string(),
        "from_state":     transition.from_state.as_str(),
        "to_state":       transition.to_state.as_str(),
        "officer_id":     transition.officer_id.map(|id| id.to_string()),
        "officer_svid":   transition.officer_svid.as_deref(),
        "transitioned_at": transition.transitioned_at.to_rfc3339(),
    });

    let canonical_bytes = serde_json::to_vec(&canonical).unwrap_or_else(|_| b"{}".to_vec());

    let mut hasher = blake3::Hasher::new();
    if !prev_hash.is_empty() {
        hasher.update(prev_hash);
    }
    hasher.update(&canonical_bytes);
    *hasher.finalize().as_bytes()
}

/// Verify the hash chain for an ordered slice of transitions (oldest first).
///
/// Returns `Ok(())` if every `this_hash` matches the recomputed value.
/// Returns `Err(ChainBroken)` at the first mismatch.
pub fn verify_chain(transitions: &[CoverStateTransition]) -> Result<(), RfidError> {
    if transitions.is_empty() {
        return Ok(());
    }

    let mut prev: &[u8] = &[];
    for t in transitions {
        let expected = compute_this_hash(prev, t);
        if t.this_hash.as_slice() != expected.as_slice() {
            return Err(RfidError::ChainBroken {
                transition_id: t.transition_id.to_string(),
                expected: hex::encode(expected),
                computed: hex::encode(&t.this_hash),
            });
        }
        prev = &t.this_hash;
    }

    Ok(())
}
