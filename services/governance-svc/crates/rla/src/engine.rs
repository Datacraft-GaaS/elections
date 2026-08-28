//! Risk-limiting audit statistics (BRAVO) and seeded sample drawing.
//!
//! Implements the BRAVO ballot-polling audit of Lindeman, Stark & Yates,
//! "BRAVO: Ballot-polling Risk-limiting Audits to Increase Confidence in
//! Elections", EVT/WOTE 2012, for plurality contests via pairwise assertions
//! (the SHANGRLA formulation of Banuelos & Stark, 2012+): the reported
//! outcome is confirmed only when every winner-over-loser assertion passes.
//!
//! Sampling is deterministic given `(manifest, seed)`: the seed is expanded
//! with SHA-256 in counter mode (NIST randomness-beacon style, per ADR-0017's
//! entropy-source model), so any party can re-derive and check the draw.

use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::models::{AuditManifest, SampledBallot};

/// One pairwise SHANGRLA/BRAVO assertion: `winner` truly has more votes than
/// `loser`. `reported_share` is the reported fraction of `{winner, loser}`
/// ballots that show `winner` (0.5 < share <= 1).
#[derive(Debug, Clone, PartialEq)]
pub struct PairwiseAssertion {
    pub winner: String,
    pub loser: String,
    pub reported_share: f64,
}

impl PairwiseAssertion {
    /// Build an assertion from raw reported totals (any third-candidate votes
    /// dilute the share automatically).
    pub fn new(
        winner: impl Into<String>,
        loser: impl Into<String>,
        winner_votes: u64,
        loser_votes: u64,
    ) -> Result<Self, RlaError> {
        let total = winner_votes + loser_votes;
        if winner_votes == 0 || total == 0 {
            return Err(RlaError::InvalidAssertion(
                "winner must have strictly more votes than loser".into(),
            ));
        }
        if winner_votes <= loser_votes {
            return Err(RlaError::InvalidAssertion(
                "assertion requires winner_votes > loser_votes".into(),
            ));
        }
        Ok(Self {
            winner: winner.into(),
            loser: loser.into(),
            reported_share: winner_votes as f64 / total as f64,
        })
    }

    /// BRAVO multiplicative update for one audited ballot.
    ///
    /// The null hypothesis is an exact tie between `winner` and `loser`
    /// (true share 0.5); the alternative is the reported share `s`. One
    /// audited ballot showing `winner` multiplies T by `s / 0.5`; one
    /// showing `loser` by `(1 - s) / 0.5`; any other ballot is
    /// uninformative and leaves T unchanged.
    pub fn update_t(&self, t: f64, audited_vote: Option<&str>) -> f64 {
        const NULL_SHARE: f64 = 0.5;
        match audited_vote {
            Some(v) if v == self.winner => t * (self.reported_share / NULL_SHARE),
            Some(v) if v == self.loser => {
                t * ((1.0 - self.reported_share) / NULL_SHARE)
            }
            _ => t,
        }
    }

    /// The risk limit threshold this assertion must reach: T >= 1/alpha.
    pub fn stop_threshold(&self, risk_limit: f64) -> f64 {
        1.0 / risk_limit.clamp(f64::EPSILON, 1.0)
    }
}

/// Engine-level errors.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RlaError {
    #[error("invalid assertion: {0}")]
    InvalidAssertion(String),
    #[error("invalid seed: {0}")]
    InvalidSeed(String),
    #[error("sample larger than manifest ({0} ballots)")]
    SampleTooLarge(u64),
}

/// Compute the BRAVO t-statistic for one assertion over an audited sample.
///
/// Ballots whose `audited_vote` is None (not yet audited) do not update T.
/// The returned value is the likelihood ratio of the data under "reported
/// outcome correct" vs "exact tie"; the audit stops when it reaches
/// `should_stop`.
pub fn compute_t_statistic(assertion: &PairwiseAssertion, ballots: &[SampledBallot]) -> f64 {
    ballots.iter().fold(1.0_f64, |t, b| {
        assertion.update_t(t, b.audited_vote.as_deref())
    })
}

/// True when every pairwise assertion has reached its stop threshold, i.e.
/// the reported outcome can be confirmed at the target risk limit.
pub fn audit_complete(
    assertions: &[PairwiseAssertion],
    ballots: &[SampledBallot],
    risk_limit: f64,
) -> bool {
    assertions
        .iter()
        .all(|a| compute_t_statistic(a, ballots) >= a.stop_threshold(risk_limit))
}

/// Alias retained for callers of the earlier API name: recount/continued
/// sampling is required while the audit has not yet confirmed the outcome.
pub fn should_trigger_recount(
    assertions: &[PairwiseAssertion],
    ballots: &[SampledBallot],
    risk_limit: f64,
) -> bool {
    !audit_complete(assertions, ballots, risk_limit)
}

/// Draw a deterministic CONTINUATION of the seeded stream behind
/// [`draw_sample_positions`]: identical algorithm, but the first `skip`
/// accepted draws are discarded and the next `take` accepted ones returned.
/// Block k of an adaptive expansion is therefore reproducible given k and
/// the sizes of all prior blocks, without re-deriving them independently.
pub fn draw_sample_block(
    manifest: &AuditManifest,
    seed: &str,
    skip: usize,
    take: usize,
) -> Result<Vec<(String, u32)>, RlaError> {
    let total: u64 = manifest.batches.iter().map(|b| b.num_ballots as u64).sum();
    if skip as u64 + take as u64 > total {
        return Err(RlaError::SampleTooLarge(total));
    }

    // Prefix sums give each global ballot index a (batch, position) home.
    let mut bounds: Vec<(u64, &crate::models::BatchEntry)> = Vec::with_capacity(manifest.batches.len());
    let mut running = 0u64;
    for b in &manifest.batches {
        running += b.num_ballots as u64;
        bounds.push((running, b));
    }

    let seed_digest = Sha256::digest(seed.as_bytes());
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(take);
    let mut counter = 0u64;
    let mut accepted = 0usize;

    while out.len() < take {
        // hash(seed_digest || counter) -> uniform global index.
        let mut h = Sha256::new();
        h.update(seed_digest);
        h.update(counter.to_be_bytes());
        counter += 1;
        let bytes = h.finalize();
        let idx = u64::from_be_bytes(bytes[..8].try_into().expect("8 bytes")) % total;

        if !seen.insert(idx) {
            continue; // rejection sampling keeps the draw without replacement
        }
        accepted += 1;
        if accepted <= skip {
            continue;
        }

        // Locate the batch holding this global index.
        let pos = bounds.partition_point(|(end, _)| *end <= idx);
        let (_, batch) = bounds[pos];
        let start = if pos == 0 { 0 } else { bounds[pos - 1].0 };
        let position = (idx - start + 1) as u32; // 1-based within batch
        out.push((batch.batch_id.clone(), position));
    }
    Ok(out)
}

/// Draw `n` ballots without replacement from the manifest, deterministically.
///
/// Returns ballot identifiers of the form `"<batch_id>/<position>"` where
/// position is 1-based within the batch. The PRNG is SHA-256 in counter mode
/// over `seed`, so the same `(manifest, seed, n)` always yields the same
/// draw -- anyone can recompute and verify it.
pub fn draw_sample_positions(
    manifest: &AuditManifest,
    seed: &str,
    n: usize,
) -> Result<Vec<(String, u32)>, RlaError> {
    draw_sample_block(manifest, seed, 0, n)
}

/// Minimum number of consecutive winner ballots needed for the BRAVO
/// t-statistic to reach this assertion's stop threshold at `risk_limit`.
///
/// Each winner ballot multiplies T by `reported_share / 0.5`, so the count
/// is `ceil((ln(threshold) - ln(current_t)) / ln(reported_share / 0.5))`.
/// Saturates at 0 once T is already past the threshold and at `u64::MAX`
/// when the assertion is unwinnable (`reported_share <= 0.5`) or T is
/// non-positive.
pub fn required_additional_samples(
    assertion: &PairwiseAssertion,
    current_t: f64,
    risk_limit: f64,
) -> u64 {
    const NULL_SHARE: f64 = 0.5;
    if current_t <= 0.0 || !current_t.is_finite() {
        return u64::MAX;
    }
    let threshold = assertion.stop_threshold(risk_limit);
    if assertion.reported_share <= NULL_SHARE {
        return u64::MAX;
    }
    if current_t >= threshold {
        return 0;
    }
    let per_winner = (assertion.reported_share / NULL_SHARE).ln();
    let needed = ((threshold.ln() - current_t.ln()) / per_winner).ceil();
    needed.max(0.0) as u64
}

/// Draw a sample and materialise it as uneaudited [`SampledBallot`] rows.
pub fn draw_sample(manifest: &AuditManifest, seed: &str, n: usize) -> Result<Vec<SampledBallot>, RlaError> {
    Ok(draw_sample_positions(manifest, seed, n)?
        .into_iter()
        .map(|(batch_id, position)| SampledBallot {
            ballot_id: format!("{batch_id}/{position}"),
            batch_id,
            position,
            manifest_entry: String::new(),
            reported_vote: None,
            audited_vote: None,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BatchEntry;

    fn assertion() -> PairwiseAssertion {
        // Reported: winner 600, loser 400 -> share 0.6.
        PairwiseAssertion::new("Ada", "Grace", 600, 400).unwrap()
    }

    #[test]
    fn rejects_non_winner_assertion() {
        assert!(PairwiseAssertion::new("A", "B", 5, 5).is_err());
        assert!(PairwiseAssertion::new("A", "B", 4, 5).is_err());
        assert!(PairwiseAssertion::new("A", "B", 0, 9).is_err());
    }

    #[test]
    fn perfect_sample_confirms_quickly() {
        let a = assertion();
        let mut t = 1.0;
        for _ in 0..50 {
            t = a.update_t(t, Some("Ada"));
        }
        assert!(t >= a.stop_threshold(0.05), "t={t}");
    }

    #[test]
    fn wrong_winner_sample_never_confirms() {
        let a = assertion();
        let mut t = 1.0;
        for _ in 0..10_000 {
            t = a.update_t(t, Some("Grace"));
        }
        assert!(t < a.stop_threshold(0.05), "wrong-winner evidence must not confirm");
    }

    #[test]
    fn non_matching_ballots_are_uninformative() {
        let a = assertion();
        assert_eq!(a.update_t(3.5, Some("other")), 3.5);
        assert_eq!(a.update_t(3.5, None), 3.5);
    }

    fn sample_ballots(votes: &[Option<&str>]) -> Vec<SampledBallot> {
        votes
            .iter()
            .enumerate()
            .map(|(i, v)| SampledBallot {
                ballot_id: format!("b{i}"),
                batch_id: "B1".into(),
                position: i as u32 + 1,
                manifest_entry: String::new(),
                reported_vote: None,
                audited_vote: v.map(str::to_string),
            })
            .collect()
    }

    #[test]
    fn end_to_end_audit_passes_on_clean_sample() {
        let a = assertion();
        let ballots = sample_ballots(&[Some("Ada"); 40]);
        assert!(!should_trigger_recount(std::slice::from_ref(&a), &ballots, 0.05));
        assert!(audit_complete(std::slice::from_ref(&a), &ballots, 0.05));
    }

    #[test]
    fn end_to_end_audit_continues_when_evidence_short() {
        let a = assertion();
        let ballots = sample_ballots(&[Some("Ada"), Some("Grace"), Some("Ada")]);
        assert!(should_trigger_recount(std::slice::from_ref(&a), &ballots, 0.05));
    }

    fn manifest(total_per_batch: u32, batches: usize) -> AuditManifest {
        AuditManifest {
            batches: (0..batches)
                .map(|i| BatchEntry {
                    batch_id: format!("BAT-{i:03}"),
                    num_ballots: total_per_batch,
                    location: format!("loc-{i}"),
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn sampling_is_deterministic_and_duplicate_free() {
        let m = manifest(100, 5); // 500 ballots
        let s1 = draw_sample_positions(&m, "beacon-pulse-42", 50).unwrap();
        let s2 = draw_sample_positions(&m, "beacon-pulse-42", 50).unwrap();
        assert_eq!(s1, s2, "same seed must reproduce the draw");
        assert_eq!(s1.len(), 50);
        let ids: HashSet<_> = s1.iter().map(|(b, p)| (b.clone(), *p)).collect();
        assert_eq!(ids.len(), 50, "draw must be without replacement");
    }

    #[test]
    fn different_seed_gives_different_draw() {
        let m = manifest(100, 5);
        let s1 = draw_sample_positions(&m, "seed-A", 20).unwrap();
        let s2 = draw_sample_positions(&m, "seed-B", 20).unwrap();
        assert_ne!(s1, s2);
    }

    #[test]
    fn sampled_ids_resolve_inside_batches() {
        let m = manifest(100, 5);
        for (batch_id, position) in draw_sample_positions(&m, "x", 30).unwrap() {
            let batch = m.batches.iter().find(|b| b.batch_id == batch_id).unwrap();
            assert!(position >= 1 && position <= batch.num_ballots);
        }
    }

    #[test]
    fn oversampling_rejected() {
        let m = manifest(10, 2); // 20 ballots
        assert_eq!(
            draw_sample_positions(&m, "s", 21),
            Err(RlaError::SampleTooLarge(20))
        );
    }

    #[test]
    fn block_continuation_matches_full_draw() {
        let m = manifest(100, 5); // 500 ballots
        let n = 60;
        let full = draw_sample_positions(&m, "beacon-pulse-42", n).unwrap();
        for k in [0usize, 1, 7, 30, 59] {
            let mut split = draw_sample_block(&m, "beacon-pulse-42", 0, k).unwrap();
            split.extend(draw_sample_block(&m, "beacon-pulse-42", k, n - k).unwrap());
            assert_eq!(split, full, "(block0 ++ block1) must equal full draw, split at {k}");
        }
    }

    #[test]
    fn multi_block_chain_matches_full_draw() {
        let m = manifest(100, 5);
        let full = draw_sample_positions(&m, "seed-A", 45).unwrap();
        let mut chained = Vec::new();
        let mut skip = 0usize;
        for take in [10usize, 20, 5, 10] {
            chained.extend(draw_sample_block(&m, "seed-A", skip, take).unwrap());
            skip += take;
        }
        assert_eq!(chained, full);
    }

    #[test]
    fn block_oversampling_and_exhaustion() {
        let m = manifest(10, 2); // 20 ballots
        assert_eq!(
            draw_sample_block(&m, "s", 15, 6),
            Err(RlaError::SampleTooLarge(20))
        );
        assert_eq!(draw_sample_block(&m, "s", 20, 0), Ok(vec![]));
        assert_eq!(draw_sample_block(&m, "s", 0, 0), Ok(vec![]));
    }

    #[test]
    fn required_additional_samples_exact_value() {
        // share 0.6 -> per-winner factor 1.2; threshold 20 (risk 0.05):
        // 1.2^16 ~= 18.49 < 20 <= 22.19 ~= 1.2^17.
        let a = assertion();
        assert_eq!(required_additional_samples(&a, 1.0, 0.05), 17);
    }

    #[test]
    fn required_additional_samples_monotone_in_t() {
        let a = assertion();
        let mut prev = u64::MAX;
        for t in [0.5f64, 1.0, 2.0, 4.0, 8.0] {
            let r = required_additional_samples(&a, t, 0.05);
            assert!(r <= prev, "required samples must not grow with t: {r} > {prev}");
            prev = r;
        }
    }

    #[test]
    fn required_additional_samples_saturates() {
        let a = assertion();
        let threshold = a.stop_threshold(0.05);
        assert_eq!(required_additional_samples(&a, threshold, 0.05), 0);
        assert_eq!(required_additional_samples(&a, threshold * 2.0, 0.05), 0);

        // Defensive: non-positive T and unwinnable shares both max out.
        assert_eq!(required_additional_samples(&a, 0.0, 0.05), u64::MAX);
        assert_eq!(required_additional_samples(&a, -3.0, 0.05), u64::MAX);
        let unwinnable = PairwiseAssertion {
            winner: "Ada".into(),
            loser: "Grace".into(),
            reported_share: 0.5,
        };
        assert_eq!(required_additional_samples(&unwinnable, 1.0, 0.05), u64::MAX);
    }

    #[test]
    fn clean_winner_block_shrinks_distance_to_threshold() {
        let a = assertion();
        let m = manifest(100, 5);
        let start = sample_ballots(&[Some("Ada"), Some("Grace")]);
        let mut t = compute_t_statistic(&a, &start);
        assert!(required_additional_samples(&a, t, 0.05) > 0);

        let block = draw_sample_block(&m, "clean-block", 0, 20).unwrap();
        for _ in &block {
            let before = required_additional_samples(&a, t, 0.05);
            t = a.update_t(t, Some("Ada"));
            let after = required_additional_samples(&a, t, 0.05);
            if before == 0 {
                assert_eq!(after, 0);
            } else {
                assert!(after < before, "clean-winner ballot must shrink distance: {after} !< {before}");
            }
        }
        assert_eq!(
            required_additional_samples(&a, t, 0.05),
            0,
            "20 consecutive winners at share 0.6 must reach any BRAVO stop threshold"
        );
    }
}
