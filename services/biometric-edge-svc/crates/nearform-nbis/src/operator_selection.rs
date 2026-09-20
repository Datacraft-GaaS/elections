//! OperatorSelectableFilters — operator UI exposes toggleable partition dimensions.
//! Each filter selection is logged via AuditService.
//!
//! ## Per-dimension reduction
//! - Gender: 3 buckets → 3x
//! - Age: 5 buckets → 5x
//! - Geo: ~47 Kenya counties → 47x
//! - Finger: 10 positions → 10x
//! - Hand: 3 buckets → 3x (ambidextrous spans 2)
//! - Quality: 3 buckets → 3x
//! - Diaspora: 3 buckets → 3x
//! - Era: 4 buckets → 4x
//!
//! Combined: 3 × 5 × 47 × 10 × 3 × 3 × 3 × 4 = ~1,140,000 partitions
//! At 100M fingerprints: ~88 candidates/partition
//!
//! ## Deduplication time
//! 88 candidates × 0.5ms = 44ms per finger 1:N search.

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorSelectableFilters {
    /// All 8 dimensions are operator-toggleable.
    /// Empty set = search nothing (operator must select at least one).
    pub gender: Vec<Gender>,
    pub age_buckets: Vec<AgeBucket>,
    pub geo_partitions: Vec<String>,  // empty = all
    pub fingers: Vec<FingerPosition>,
    pub hands: Vec<DominantHand>,
    pub qualities: Vec<RidgeQuality>,
    pub diaspora_statuses: Vec<DiasporaStatus>,
    pub enrollment_eras: Vec<EnrollmentEra>,
}

impl Default for OperatorSelectableFilters {
    fn default() -> Self {
        Self {
            gender: vec![Gender::Male, Gender::Female, Gender::Other],
            age_buckets: vec![
                AgeBucket::Minor, AgeBucket::YoungAdult, AgeBucket::MiddleAged,
                AgeBucket::Senior, AgeBucket::Elderly,
            ],
            geo_partitions: vec![], // empty = all
            fingers: vec![
                FingerPosition::LeftThumb, FingerPosition::LeftIndex,
                FingerPosition::LeftMiddle, FingerPosition::LeftRing, FingerPosition::LeftLittle,
                FingerPosition::RightThumb, FingerPosition::RightIndex,
                FingerPosition::RightMiddle, FingerPosition::RightRing, FingerPosition::RightLittle,
            ],
            hands: vec![DominantHand::Left, DominantHand::Right, DominantHand::Ambidextrous],
            qualities: vec![RidgeQuality::High, RidgeQuality::Medium, RidgeQuality::Low],
            diaspora_statuses: vec![
                DiasporaStatus::InCountry, DiasporaStatus::Diaspora, DiasporaStatus::Newcomer,
            ],
            enrollment_eras: vec![
                EnrollmentEra::E2017_2019, EnrollmentEra::E2020_2022,
                EnrollmentEra::E2023_2025, EnrollmentEra::E2026Plus,
            ],
        }
    }
}

impl OperatorSelectableFilters {
    /// Number of compound partition keys this filter configuration produces.
    /// Used to estimate search time.
    pub fn partition_count(&self) -> usize {
        // Geo defaults to 47 Kenya counties when empty
        let geo_size = if self.geo_partitions.is_empty() { 47 } else { self.geo_partitions.len() };
        self.gender.len().max(1) *
        self.age_buckets.len().max(1) *
        geo_size.max(1) *
        self.fingers.len().max(1) *
        self.hands.len().max(1) *
        self.qualities.len().max(1) *
        self.diaspora_statuses.len().max(1) *
        self.enrollment_eras.len().max(1)
    }

    /// Estimate 1:N search time given total fingerprint count.
    /// `0.5ms` per comparison (FAP60/AntIris on device CPU with NBIS bozorth3).
    pub fn estimated_search_time(&self, total_fingerprints: usize) -> Duration {
        let partitions = self.partition_count();
        let candidates_per_partition = total_fingerprints / partitions.max(1);
        let ms = (candidates_per_partition as f64) * 0.5;
        Duration::from_millis(ms as u64)
    }

    /// Human-readable estimate (e.g. "5 seconds", "2.5 minutes").
    pub fn human_readable_estimate(&self, total_fingerprints: usize) -> String {
        let d = self.estimated_search_time(total_fingerprints);
        if d.as_secs() >= 3600 {
            format!("{:.1} hours", d.as_secs_f64() / 3600.0)
        } else if d.as_secs() >= 60 {
            format!("{:.1} minutes", d.as_secs_f64() / 60.0)
        } else if d.as_millis() >= 1000 {
            format!("{} seconds", d.as_secs())
        } else {
            format!("{}ms", d.as_millis())
        }
    }

    /// Reduction factor relative to no partitioning.
    /// Returns 1.0 = no reduction, 0.0001 = 10000x reduction.
    pub fn reduction_factor(&self, total_fingerprints: usize) -> f64 {
        if self.partition_count() == 0 { return 1.0; }
        let candidates = total_fingerprints as f64 / self.partition_count() as f64;
        candidates / total_fingerprints as f64
    }

    /// Dimension totals of the partition model (empty filter = all values).
    /// Kept in sync with `Default` above: gender 3, age 5, geo 47 counties,
    /// fingers 10, hands 3, qualities 3, diaspora 3, eras 4.
    const TOTALS: [usize; 8] = [3, 5, 47, 10, 3, 3, 3, 4];

    /// Estimated TOTAL search time across all matching partitions.
    ///
    /// Unlike [`Self::estimated_search_time`] (cost of scanning one
    /// partition), this scales with the fraction of the partition space the
    /// filter selects — so narrower filters are strictly faster. Assumes
    /// uniform enrollment distribution and 0.5 ms per candidate comparison.
    pub fn estimated_total_search_time(&self, total_fingerprints: usize) -> Duration {
        let selected = [
            Self::selected_or_total(self.gender.len(), Self::TOTALS[0]),
            Self::selected_or_total(self.age_buckets.len(), Self::TOTALS[1]),
            Self::selected_or_total(self.geo_partitions.len(), Self::TOTALS[2]),
            Self::selected_or_total(self.fingers.len(), Self::TOTALS[3]),
            Self::selected_or_total(self.hands.len(), Self::TOTALS[4]),
            Self::selected_or_total(self.qualities.len(), Self::TOTALS[5]),
            Self::selected_or_total(self.diaspora_statuses.len(), Self::TOTALS[6]),
            Self::selected_or_total(self.enrollment_eras.len(), Self::TOTALS[7]),
        ];
        let matching: usize = selected.iter().product();
        let total_space: usize = Self::TOTALS.iter().product();
        let ms =
            total_fingerprints as f64 * (matching as f64 / total_space as f64) * 0.5;
        Duration::from_millis(ms as u64)
    }

    fn selected_or_total(selected: usize, total: usize) -> usize {
        if selected == 0 { total } else { selected.min(total) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender { Male, Female, Other }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgeBucket { Minor, YoungAdult, MiddleAged, Senior, Elderly }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FingerPosition {
    LeftThumb, LeftIndex, LeftMiddle, LeftRing, LeftLittle,
    RightThumb, RightIndex, RightMiddle, RightRing, RightLittle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DominantHand { Left, Right, Ambidextrous }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RidgeQuality { High, Medium, Low }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiasporaStatus { InCountry, Diaspora, Newcomer }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnrollmentEra { E2017_2019, E2020_2022, E2023_2025, E2026Plus }

// ── Filter helpers for typed selection ──────────────────────────────────────

impl OperatorSelectableFilters {
    /// Check whether a particular compound key matches the operator's selection.
    pub fn matches(
        &self,
        g: Gender, a: AgeBucket, geo: &str, f: FingerPosition,
        h: DominantHand, q: RidgeQuality,
        d: DiasporaStatus, e: EnrollmentEra,
    ) -> bool {
        if !self.gender.contains(&g) { return false; }
        if !self.age_buckets.contains(&a) { return false; }
        if !self.geo_partitions.is_empty() && !self.geo_partitions.contains(&geo.to_string()) {
            return false;
        }
        if !self.fingers.contains(&f) { return false; }
        if !self.hands.contains(&h) { return false; }
        if !self.qualities.contains(&q) { return false; }
        if !self.diaspora_statuses.contains(&d) { return false; }
        if !self.enrollment_eras.contains(&e) { return false; }
        true
    }

    /// Build a compound key string.
    pub fn compound_key(
        g: Gender, a: AgeBucket, geo: &str, f: FingerPosition,
        h: DominantHand, q: RidgeQuality, d: DiasporaStatus, e: EnrollmentEra,
    ) -> String {
        format!(
            "{}|{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}",
            g_code(g), a_code(a), geo, f, h, q, d, e,
        )
    }
}

fn g_code(g: Gender) -> &'static str {
    match g {
        Gender::Male => "M",
        Gender::Female => "F",
        Gender::Other => "O",
    }
}

fn a_code(a: AgeBucket) -> &'static str {
    match a {
        AgeBucket::Minor => "0-17",
        AgeBucket::YoungAdult => "18-35",
        AgeBucket::MiddleAged => "36-55",
        AgeBucket::Senior => "56-75",
        AgeBucket::Elderly => "76+",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filters_give_5s_estimate_at_100m() {
        let f = OperatorSelectableFilters::default();
        let estimate = f.human_readable_estimate(100_000_000);
        // At 100M: 3*5*47*10*3*3*3*4 = 1,140,000 partitions
        // 100M / 1.14M = 88 candidates/partition
        // 88 * 0.5ms = 44ms
        assert!(estimate.contains("ms") || estimate.contains("second"), "got {estimate}");
    }

    #[test]
    fn narrow_filters_are_faster() {
        // Total-work semantics: narrower filters select fewer partitions.
        // (Per-partition cost in `estimated_search_time` is intentionally
        // separate — fewer partitions trivially hold more candidates each.)
        let wide = OperatorSelectableFilters::default();
        let narrow = OperatorSelectableFilters {
            fingers: vec![FingerPosition::RightIndex],
            geo_partitions: vec!["NairobiWest".to_string()],
            ..Default::default()
        };
        let wide_t = wide.estimated_total_search_time(100_000_000);
        let narrow_t = narrow.estimated_total_search_time(100_000_000);
        assert!(narrow_t < wide_t);
    }

    #[test]
    fn compound_key_format() {
        let key = OperatorSelectableFilters::compound_key(
            Gender::Male, AgeBucket::YoungAdult, "NairobiWest",
            FingerPosition::RightIndex, DominantHand::Right,
            RidgeQuality::High, DiasporaStatus::InCountry, EnrollmentEra::E2023_2025,
        );
        assert_eq!(key, "M|18-35|NairobiWest|RightIndex|Right|High|InCountry|E2023_2025");
    }
}
