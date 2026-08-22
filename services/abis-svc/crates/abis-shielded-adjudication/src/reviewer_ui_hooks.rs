//! Hooks for the Django adjudicator console to render shielded-only side-by-side.
//!
//! The UI receives occlusion + intersection counts rather than raw images or
//! templates. This prevents any plaintext biometric data from reaching the
//! reviewer's browser.

use serde::{Deserialize, Serialize};

use crate::case_shield::ShieldedCase;
use crate::tier_gate::{allows_plaintext, ReviewMode};

/// Payload sent to the Django adjudicator console for one candidate pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedReviewPayload {
    pub case_id: String,
    pub gallery_index: usize,
    /// Jaccard similarity score in [0, 1].
    pub similarity: f64,
    /// Number of bits set in (probe XOR gallery) -- visualises occlusion.
    pub occlusion_bits: usize,
    /// Number of bits set in (probe AND gallery) -- visualises intersection.
    pub intersection_bits: usize,
    /// True if this pair is above the match threshold.
    pub above_threshold: bool,
    /// Review mode for this case; determines what the UI may render.
    pub review_mode: ReviewMode,
}

/// Build the reviewer payload for one case + gallery index.
///
/// Returns `None` if `gallery_index` is out of range.
pub fn build_payload(
    case: &ShieldedCase,
    gallery_index: usize,
    mode: ReviewMode,
) -> Option<ShieldedReviewPayload> {
    let (occlusion_bits, intersection_bits) = case.reviewer_visual_hints(gallery_index)?;
    let similarity = *case.similarity_scores.get(gallery_index)?;
    let above_threshold = case.above_threshold;
    Some(ShieldedReviewPayload {
        case_id: case.case_id.clone(),
        gallery_index,
        similarity,
        occlusion_bits,
        intersection_bits,
        above_threshold,
        review_mode: mode,
    })
}

/// Assert that the payload contains no raw template data.
///
/// Called by the audit layer before the payload is serialised for the wire.
/// Returns `Err` with a descriptive message if a leak is detected.
pub fn assert_no_template_leak(
    payload: &ShieldedReviewPayload,
    probe_hex: &str,
) -> Result<(), String> {
    let json = serde_json::to_string(payload).unwrap_or_default();
    if json.contains(probe_hex) {
        return Err(format!(
            "template leak detected in review payload for case {}",
            payload.case_id
        ));
    }
    Ok(())
}

/// Returns `true` if the current mode permits the UI to render raw images.
///
/// The Django template uses this to gate the `<img>` tag rendering.
pub fn may_render_raw_image(mode: ReviewMode) -> bool {
    allows_plaintext(mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_shield::ShieldedCase;
    use crate::tier_gate::{gate, CaseSeverity, ReviewMode};

    fn make_case() -> ShieldedCase {
        let probe = vec![0xAA; 32];
        let gallery = vec![vec![0xAA; 32], vec![0x55; 32]];
        let gallery_refs: Vec<&[u8]> = gallery.iter().map(|v| v.as_slice()).collect();
        ShieldedCase::new("ui-001".to_string(), &probe, &gallery_refs, 512, 3, 0.5).unwrap()
    }

    #[test]
    fn payload_built_for_gallery_0() {
        let case = make_case();
        let mode = gate(CaseSeverity::Tier3Routine);
        let payload = build_payload(&case, 0, mode).unwrap();
        assert_eq!(payload.case_id, "ui-001");
        assert_eq!(payload.gallery_index, 0);
        assert!(payload.similarity >= 0.0 && payload.similarity <= 1.0);
    }

    #[test]
    fn payload_out_of_range_returns_none() {
        let case = make_case();
        let mode = gate(CaseSeverity::Tier3Routine);
        assert!(build_payload(&case, 99, mode).is_none());
    }

    #[test]
    fn shielded_mode_does_not_render_raw_image() {
        assert!(!may_render_raw_image(ReviewMode::ShieldedReview));
    }

    #[test]
    fn plaintext_mode_renders_raw_image() {
        assert!(may_render_raw_image(ReviewMode::PlaintextDualReview));
    }

    #[test]
    fn no_template_leak_in_payload() {
        let case = make_case();
        let mode = gate(CaseSeverity::Tier2High);
        let payload = build_payload(&case, 0, mode).unwrap();
        let probe_hex = hex::encode(vec![0xAA; 32]);
        assert!(assert_no_template_leak(&payload, &probe_hex).is_ok());
    }
}
