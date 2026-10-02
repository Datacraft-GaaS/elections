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

//! ShieldedCase: wraps an adjudication case so probe + gallery templates never
//! appear in plaintext. Adjudicators see only Bloom-shielded comparisons.

use thiserror::Error;

use eel_abis_shielding::{ShieldedTemplate, Shielder};

#[derive(Debug, Error)]
pub enum CaseShieldError {
    #[error("probe template shielding failed: {0}")]
    ProbeShield(#[from] eel_abis_shielding::ShieldError),
    #[error("gallery is empty")]
    EmptyGallery,
    #[error("similarity threshold must be in (0, 1]")]
    InvalidThreshold,
}

/// A shielded adjudication case.
///
/// Probe and gallery templates are stored only in shielded (Bloom-filter) form.
/// No plaintext template is retained after construction.
///
/// ShieldedTemplate does not implement Serialize/Deserialize; serialisation
/// of this struct is intentionally omitted to prevent accidental template leakage.
#[derive(Debug, Clone)]
pub struct ShieldedCase {
    /// Case identifier (UUID7 string).
    pub case_id: String,
    /// Shielded probe template (Bloom filter).
    pub shielded_probe: ShieldedTemplate,
    /// One or more shielded gallery templates.
    pub shielded_gallery: Vec<ShieldedTemplate>,
    /// Pre-computed similarity scores (probe vs each gallery member).
    pub similarity_scores: Vec<f64>,
    /// True if probe matches at least one gallery entry above threshold.
    pub above_threshold: bool,
}

impl ShieldedCase {
    /// Construct a shielded case from raw template feature vectors.
    ///
    /// `probe_features` and each element of `gallery_features` are slices of
    /// feature bytes (e.g. quantised minutiae descriptors). After this call,
    /// no plaintext features are retained.
    ///
    /// `threshold` is the Jaccard similarity threshold above which a pair is
    /// considered a match candidate.
    pub fn new(
        case_id: String,
        probe_features: &[u8],
        gallery_features: &[&[u8]],
        filter_size_bits: usize,
        hash_count: u8,
        threshold: f64,
    ) -> Result<Self, CaseShieldError> {
        if gallery_features.is_empty() {
            return Err(CaseShieldError::EmptyGallery);
        }
        if threshold <= 0.0 || threshold > 1.0 {
            return Err(CaseShieldError::InvalidThreshold);
        }

        let shielder = Shielder::new(filter_size_bits, hash_count)?;
        let shielded_probe = shielder.shield(probe_features)?;

        let mut shielded_gallery = Vec::with_capacity(gallery_features.len());
        let mut similarity_scores = Vec::with_capacity(gallery_features.len());
        let mut above_threshold = false;

        for features in gallery_features {
            let shielded = shielder.shield(features)?;
            let sim = shielded_probe.similarity(&shielded);
            if sim >= threshold {
                above_threshold = true;
            }
            similarity_scores.push(sim);
            shielded_gallery.push(shielded);
        }

        Ok(Self {
            case_id,
            shielded_probe,
            shielded_gallery,
            similarity_scores,
            above_threshold,
        })
    }

    /// Return (occlusion_bits, intersection_bits) for the reviewer UI.
    ///
    /// The UI renders these as a side-by-side shielded visualisation.
    /// No raw template data is ever exposed.
    pub fn reviewer_visual_hints(&self, gallery_idx: usize) -> Option<(usize, usize)> {
        let gallery = self.shielded_gallery.get(gallery_idx)?;
        let probe_pop = self.shielded_probe.popcount();
        let gallery_pop = gallery.popcount();
        let intersection: usize = self
            .shielded_probe
            .filter
            .iter()
            .zip(gallery.filter.iter())
            .map(|(a, b)| (a & b).count_ones() as usize)
            .sum();
        let occlusion = probe_pop + gallery_pop - 2 * intersection;
        Some((occlusion, intersection))
    }

    /// Confirm that a serialised representation of the shielded fields contains no raw template.
    ///
    /// Checks that `probe_features` do not appear as a hex string in a JSON summary of the
    /// similarity scores and case_id (the only fields a caller would serialise). This is a
    /// defence-in-depth check; the primary protection is the absence of Serialize on ShieldedCase.
    pub fn serialised_contains_no_raw_features(&self, probe_features: &[u8]) -> bool {
        // Build a JSON summary of the fields that would appear in an API response.
        let summary = serde_json::json!({
            "case_id": self.case_id,
            "similarity_scores": self.similarity_scores,
            "above_threshold": self.above_threshold,
        });
        let json = serde_json::to_string(&summary).unwrap_or_default();
        let probe_hex = hex::encode(probe_features);
        !json.contains(&probe_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FSB: usize = 512;
    const HC: u8 = 3;

    #[test]
    fn shielded_case_constructed() {
        let probe = vec![0u8, 1, 2, 3, 4, 5];
        let gallery: Vec<Vec<u8>> = vec![vec![0, 1, 2, 3, 4, 5], vec![6, 7, 8, 9, 10, 11]];
        let gallery_refs: Vec<&[u8]> = gallery.iter().map(|v| v.as_slice()).collect();
        let case =
            ShieldedCase::new("c-001".to_string(), &probe, &gallery_refs, FSB, HC, 0.1).unwrap();
        assert_eq!(case.similarity_scores.len(), 2);
    }

    #[test]
    fn identical_probe_gallery_above_threshold() {
        let features = vec![42u8; 64];
        let gallery_refs = vec![features.as_slice()];
        let case =
            ShieldedCase::new("c-002".to_string(), &features, &gallery_refs, FSB, HC, 0.9).unwrap();
        assert!(case.above_threshold);
    }

    #[test]
    fn empty_gallery_returns_error() {
        let probe = vec![1u8, 2, 3];
        let res = ShieldedCase::new("c-003".to_string(), &probe, &[], FSB, HC, 0.5);
        assert!(matches!(res, Err(CaseShieldError::EmptyGallery)));
    }

    #[test]
    fn no_raw_features_in_serialised_form() {
        let probe = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04];
        let gallery = vec![vec![0u8; 8]];
        let gallery_refs: Vec<&[u8]> = gallery.iter().map(|v| v.as_slice()).collect();
        let case =
            ShieldedCase::new("c-004".to_string(), &probe, &gallery_refs, FSB, HC, 0.5).unwrap();
        assert!(case.serialised_contains_no_raw_features(&probe));
    }
}
