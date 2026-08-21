//! IREX 1:N identification track.

use crate::{corpus::CorpusEntry, verify::IrexVerifier};
use serde::{Deserialize, Serialize};

/// Result of a 1:N identification search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyResult {
    pub probe_id: String,
    /// ID of the top-ranked gallery entry.
    pub rank1_id: String,
    /// Score of the top-ranked gallery entry.
    pub rank1_score: f32,
    /// Ordered list of (id, score) for the top-N candidates.
    pub rank_n: Vec<(String, f32)>,
}

/// 1:N identifier that returns the top-N scoring gallery entries.
#[derive(Debug, Clone)]
pub struct IrexIdentifier {
    /// Number of candidates to return.
    pub n: usize,
    pub verifier: IrexVerifier,
}

impl IrexIdentifier {
    /// Create an identifier returning the top `n` candidates.
    pub fn new(n: usize, threshold: f32) -> Self {
        Self {
            n,
            verifier: IrexVerifier::new(threshold),
        }
    }

    /// Score `probe` against every `gallery` entry, returning the top-N ranked results.
    ///
    /// If `gallery` has fewer than `n` entries all are returned.
    pub fn identify(&self, probe: &CorpusEntry, gallery: &[CorpusEntry]) -> IdentifyResult {
        let mut scored: Vec<(String, f32)> = gallery
            .iter()
            .map(|g| {
                let r = self.verifier.verify(probe, g);
                (g.id.clone(), r.score)
            })
            .collect();

        // Sort descending by score.
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let rank_n: Vec<(String, f32)> = scored.into_iter().take(self.n).collect();

        let (rank1_id, rank1_score) = rank_n
            .first()
            .map(|(id, s)| (id.clone(), *s))
            .unwrap_or_else(|| (String::new(), 0.0));

        IdentifyResult {
            probe_id: probe.id.clone(),
            rank1_id,
            rank1_score,
            rank_n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_encode::{
        daugman_code::DaugmanCode, pie::PieCode, IrisTemplate, IRIS_CODE_BYTES,
    };

    fn synthetic_template(seed: u8) -> IrisTemplate {
        let code: [u8; IRIS_CODE_BYTES] = std::array::from_fn(|_| seed);
        let mask: [u8; IRIS_CODE_BYTES] = [0xFF; IRIS_CODE_BYTES];
        IrisTemplate {
            daugman: DaugmanCode { code, mask },
            pie: PieCode { code, mask },
            quality: seed as f32 / 255.0,
        }
    }

    fn make_entry(id: &str, seed: u8) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            template: synthetic_template(seed),
            group: "g1".to_string(),
            quality: seed as f32 / 255.0,
        }
    }

    #[test]
    fn rank1_is_highest_scored() {
        let identifier = IrexIdentifier::new(3, 0.5);
        let probe = make_entry("probe_1", 50);
        let gallery = vec![
            make_entry("g_a", 51), // closest seed to probe
            make_entry("g_b", 0),
            make_entry("g_c", 255),
        ];
        let result = identifier.identify(&probe, &gallery);
        // rank1 should have the highest score among the three
        let r1_score = result.rank1_score;
        for (_, s) in &result.rank_n {
            assert!(*s <= r1_score + 1e-6, "rank1 should be highest");
        }
    }

    #[test]
    fn rank_n_capped_at_n() {
        let identifier = IrexIdentifier::new(2, 0.5);
        let probe = make_entry("probe_1", 50);
        let gallery = vec![
            make_entry("g_a", 10),
            make_entry("g_b", 20),
            make_entry("g_c", 30),
            make_entry("g_d", 40),
        ];
        let result = identifier.identify(&probe, &gallery);
        assert_eq!(result.rank_n.len(), 2);
    }

    #[test]
    fn rank_n_smaller_than_n_when_gallery_small() {
        let identifier = IrexIdentifier::new(5, 0.5);
        let probe = make_entry("probe_1", 50);
        let gallery = vec![make_entry("g_a", 10), make_entry("g_b", 20)];
        let result = identifier.identify(&probe, &gallery);
        assert!(result.rank_n.len() <= 2);
    }

    #[test]
    fn rank_n_ordered_descending() {
        let identifier = IrexIdentifier::new(4, 0.0);
        let probe = make_entry("probe_1", 100);
        let gallery = vec![
            make_entry("g_a", 10),
            make_entry("g_b", 100),
            make_entry("g_c", 50),
            make_entry("g_d", 200),
        ];
        let result = identifier.identify(&probe, &gallery);
        let scores: Vec<f32> = result.rank_n.iter().map(|(_, s)| *s).collect();
        for i in 1..scores.len() {
            assert!(scores[i - 1] >= scores[i], "scores not descending");
        }
    }
}
