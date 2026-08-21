//! Fairness harness: per-demographic FMR/FNMR slices and equalised-odds breach detection.
//!
//! Extends the core fmr_fnmr.rs with demographic disaggregation (ISO/IEC 19795-10).
//! Tolerances: ≤5% variation across sex, ≤10% across age buckets.

// Re-export shared demographic types from the index layer.
pub use crate::index::bloom_prune::{AgeBucket, DemographicKey, Sex};

/// Per-slice accuracy metrics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FairnessSlice {
    pub demographic: DemographicKey,
    /// False match rate for this slice.
    pub fmr: f64,
    /// False non-match rate for this slice.
    pub fnmr: f64,
    pub sample_size: usize,
}

/// A recorded match attempt with ground-truth label.
#[derive(Debug, Clone)]
pub struct MatchRecord {
    pub demographic: DemographicKey,
    /// True = genuine pair (should match), False = impostor.
    pub is_genuine: bool,
    /// Whether the system returned a match.
    pub system_match: bool,
}

/// A detected equalised-odds breach.
#[derive(Debug, Clone)]
pub struct FairnessBreach {
    pub metric: FairnessMetric,
    pub slice_a: DemographicKey,
    pub slice_b: DemographicKey,
    pub value_a: f64,
    pub value_b: f64,
    pub delta: f64,
    pub tolerance: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FairnessMetric {
    Fmr,
    Fnmr,
}

/// Compute fairness slices and detect breaches.
pub struct FairnessReport;

impl FairnessReport {
    /// Compute per-demographic FMR/FNMR slices from a set of match records.
    pub fn compute(records: &[MatchRecord]) -> Vec<FairnessSlice> {
        use std::collections::HashMap;

        #[derive(Default)]
        struct Counts {
            genuine_total: usize,
            genuine_match: usize,
            impostor_total: usize,
            impostor_match: usize,
        }

        let mut by_demo: HashMap<String, (DemographicKey, Counts)> = HashMap::new();

        for r in records {
            let key = format!(
                "{}|{}|{}",
                match r.demographic.sex {
                    Sex::M => "M",
                    Sex::F => "F",
                },
                match r.demographic.age_bucket {
                    AgeBucket::Age18_25 => "18_25",
                    AgeBucket::Age26_40 => "26_40",
                    AgeBucket::Age41_60 => "41_60",
                    AgeBucket::Age61Plus => "61_plus",
                },
                r.demographic.county,
            );
            let entry = by_demo
                .entry(key)
                .or_insert_with(|| (r.demographic.clone(), Counts::default()));
            let counts = &mut entry.1;
            if r.is_genuine {
                counts.genuine_total += 1;
                if r.system_match {
                    counts.genuine_match += 1;
                }
            } else {
                counts.impostor_total += 1;
                if r.system_match {
                    counts.impostor_match += 1;
                }
            }
        }

        by_demo
            .into_values()
            .map(|(demo, c)| {
                let fmr = if c.impostor_total > 0 {
                    c.impostor_match as f64 / c.impostor_total as f64
                } else {
                    0.0
                };
                let fnmr = if c.genuine_total > 0 {
                    let misses = c.genuine_total - c.genuine_match;
                    misses as f64 / c.genuine_total as f64
                } else {
                    0.0
                };
                FairnessSlice {
                    demographic: demo,
                    fmr,
                    fnmr,
                    sample_size: c.genuine_total + c.impostor_total,
                }
            })
            .collect()
    }

    /// Detect equalised-odds breaches across sex (5% tolerance) and age bucket (10% tolerance).
    pub fn detect_breaches(slices: &[FairnessSlice]) -> Vec<FairnessBreach> {
        let mut breaches = Vec::new();

        for i in 0..slices.len() {
            for j in (i + 1)..slices.len() {
                let a = &slices[i];
                let b = &slices[j];

                let same_county = a.demographic.county == b.demographic.county;
                let same_age = a.demographic.age_bucket == b.demographic.age_bucket;
                let diff_sex = a.demographic.sex != b.demographic.sex;

                if same_county && same_age && diff_sex {
                    check_breach(a, b, FairnessMetric::Fmr, 0.05, &mut breaches);
                    check_breach(a, b, FairnessMetric::Fnmr, 0.05, &mut breaches);
                }

                let diff_age = a.demographic.age_bucket != b.demographic.age_bucket;
                let same_sex = a.demographic.sex == b.demographic.sex;

                if same_county && same_sex && diff_age {
                    check_breach(a, b, FairnessMetric::Fmr, 0.10, &mut breaches);
                    check_breach(a, b, FairnessMetric::Fnmr, 0.10, &mut breaches);
                }
            }
        }
        breaches
    }
}

fn check_breach(
    a: &FairnessSlice,
    b: &FairnessSlice,
    metric: FairnessMetric,
    tolerance: f64,
    out: &mut Vec<FairnessBreach>,
) {
    let (va, vb) = match metric {
        FairnessMetric::Fmr => (a.fmr, b.fmr),
        FairnessMetric::Fnmr => (a.fnmr, b.fnmr),
    };
    let delta = (va - vb).abs();
    if delta > tolerance {
        out.push(FairnessBreach {
            metric,
            slice_a: a.demographic.clone(),
            slice_b: b.demographic.clone(),
            value_a: va,
            value_b: vb,
            delta,
            tolerance,
        });
    }
}
