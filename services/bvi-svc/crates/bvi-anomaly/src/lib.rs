//! Election-night statistical anomaly detection (pure functions).
//!
//! All detectors are side-effect-free: they take plain data and return
//! serializable findings. Nothing here touches a database or the network,
//! so every function is directly unit-testable.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity attached to every anomaly finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    High,
    Critical,
}

/// A single detected anomaly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnoutAnomaly {
    pub kind: String,
    pub station_code: String,
    pub detail: String,
    pub severity: Severity,
}

/// A device observation at a station, used for impossible-velocity checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceRead {
    pub device_serial: String,
    pub station_code: String,
    pub seen_at: DateTime<Utc>,
}

/// Result of a digit-distribution chi-square test.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DigitStats {
    pub chi_square: f64,
    pub dof: f64,
    pub p_value: f64,
}

/// Detector (a): more voters identified than registered at a station.
///
/// Fires when `identified > registered` and `registered >= 0`. Returns `None`
/// for negative registered rolls (malformed input) or non-anomalous data.
pub fn turnover_over_registered(
    station_code: impl Into<String>,
    identified: i64,
    registered: i64,
) -> Option<TurnoutAnomaly> {
    if registered < 0 || identified <= registered {
        return None;
    }
    Some(TurnoutAnomaly {
        kind: "turnover_over_registered".to_string(),
        station_code: station_code.into(),
        detail: format!("identified ({identified}) exceeds registered roll ({registered})"),
        severity: Severity::Critical,
    })
}

/// Detector (b): a single device seen at two different stations closer
/// together than `min_travel_seconds` allows.
///
/// Reads are grouped by `device_serial` and ordered by time; each consecutive
/// pair at *different* stations with a gap below the threshold produces one
/// High-severity anomaly naming both stations.
pub fn velocity_impossible(reads: &[DeviceRead], min_travel_seconds: i64) -> Vec<TurnoutAnomaly> {
    let mut by_device: BTreeMap<&str, Vec<&DeviceRead>> = BTreeMap::new();
    for read in reads {
        by_device
            .entry(read.device_serial.as_str())
            .or_default()
            .push(read);
    }

    let mut anomalies = Vec::new();
    for (serial, mut group) in by_device {
        group.sort_by_key(|r| r.seen_at);
        for pair in group.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if a.station_code == b.station_code {
                continue;
            }
            let gap = (b.seen_at - a.seen_at).num_seconds();
            if gap < min_travel_seconds {
                anomalies.push(TurnoutAnomaly {
                    kind: "velocity_impossible".to_string(),
                    station_code: b.station_code.clone(),
                    detail: format!(
                        "device {serial} moved from {} to {} in {gap}s (min travel time {min_travel_seconds}s)",
                        a.station_code, b.station_code
                    ),
                    severity: Severity::High,
                });
            }
        }
    }
    anomalies
}

/// Detector (c): one window's turnout is an outlier vs all other windows.
///
/// Computes mean/stdev over every window *except* `window_index`; a z-score
/// above `threshold` fires a Critical anomaly. Returns `None` for fewer than
/// 3 windows, an out-of-range index, or zero stdev (nothing to compare).
pub fn burst_zscore(
    series: &[f64],
    window_index: usize,
    window_len: usize,
    threshold: f64,
) -> Option<TurnoutAnomaly> {
    if series.len() < 3 || window_index >= series.len() || window_len == 0 {
        return None;
    }
    let tested = series[window_index];
    let rest: Vec<f64> = series
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != window_index)
        .map(|(_, v)| *v)
        .collect();

    let n = rest.len() as f64;
    let mean = rest.iter().sum::<f64>() / n;
    let variance = rest.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let stdev = variance.sqrt();
    if stdev == 0.0 {
        return None;
    }

    let z = (tested - mean) / stdev;
    if z > threshold {
        Some(TurnoutAnomaly {
            kind: "burst_zscore".to_string(),
            station_code: format!("window:{window_index}/{window_len}"),
            detail: format!(
                "turnout burst z={z:.3} exceeds threshold {threshold} (baseline mean={mean:.3}, stdev={stdev:.3})"
            ),
            severity: Severity::Critical,
        })
    } else {
        None
    }
}

/// Detector (d): chi-square goodness-of-fit of observed counts against
/// `expected_shares`.
///
/// Validates equal lengths and non-negative shares; degrees of freedom is
/// k-1. The p-value uses the Wilson-Hilferty cube-root normal approximation
/// of the chi-square CDF.
pub fn digit_distribution_chi_square(
    counts: &[u64],
    expected_shares: &[f64],
) -> Result<DigitStats, String> {
    if counts.len() != expected_shares.len() {
        return Err(format!(
            "counts length ({}) must match expected_shares length ({})",
            counts.len(),
            expected_shares.len()
        ));
    }
    if counts.is_empty() {
        return Err("at least one category is required".to_string());
    }
    if expected_shares.iter().any(|s| *s < 0.0) {
        return Err("expected_shares must be non-negative".to_string());
    }

    let total = counts.iter().sum::<u64>();
    if total == 0 {
        return Ok(DigitStats {
            chi_square: 0.0,
            dof: (counts.len() - 1) as f64,
            p_value: 1.0,
        });
    }

    let chi2: f64 = counts
        .iter()
        .zip(expected_shares)
        .map(|(&c, &s)| {
            let expected = s * total as f64;
            if expected > 0.0 {
                let diff = c as f64 - expected;
                diff * diff / expected
            } else {
                // Observed count under a zero-probability category is fully anomalous.
                if c > 0 {
                    f64::INFINITY
                } else {
                    0.0
                }
            }
        })
        .sum();

    let dof = (counts.len() - 1) as f64;
    let p_value = chi_square_sf_upper(chi2, dof);

    Ok(DigitStats {
        chi_square: chi2,
        dof,
        p_value,
    })
}

/// Wilson-Hilferty approximation of the upper-tail probability
/// P(X > x) for X ~ chi-square(df): transform to an approximately standard
/// normal z and return 1 - Phi(z).
fn chi_square_sf_upper(x: f64, df: f64) -> f64 {
    if !x.is_finite() {
        return 0.0;
    }
    if x <= 0.0 || df <= 0.0 {
        return 1.0;
    }
    let t = (x / df).cbrt();
    let mu = 1.0 - 2.0 / (9.0 * df);
    let sigma = (2.0 / (9.0 * df)).sqrt();
    let z = (t - mu) / sigma;
    1.0 - standard_normal_cdf(z)
}

/// Standard normal CDF via erfc (Abramowitz & Stegun 7.1.26, |err| < 1.5e-7).
fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * erfc(-z / std::f64::consts::SQRT_2)
}

/// Complementary error function approximation (A&S 7.1.26, |err| < 1.5e-7).
fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let poly = t
        * (1.000_023_68
            + t * (0.374_091_96
                + t * (0.096_784_18
                    + t * (-0.186_288_06
                        + t * (0.278_868_07
                            + t * (-1.135_203_98
                                + t * (1.488_515_87
                                    + t * (-0.822_152_23 + t * 0.170_872_77))))))));
    let val = t * (-z * z - 1.265_512_23 + poly).exp();

    if x >= 0.0 {
        val
    } else {
        2.0 - val
    }
}

/// Convenience combination of detectors (a) and (c) for one station.
pub fn evaluate_station(
    station_code: &str,
    identified: i64,
    registered: i64,
    series: &[f64],
    window_index: usize,
    window_len: usize,
    z_threshold: f64,
) -> Vec<TurnoutAnomaly> {
    let mut anomalies = Vec::new();
    if let Some(a) = turnover_over_registered(station_code, identified, registered) {
        anomalies.push(a);
    }
    if let Some(b) = burst_zscore(series, window_index, window_len, z_threshold) {
        let b = TurnoutAnomaly {
            station_code: station_code.to_string(),
            ..b
        };
        anomalies.push(b);
    }
    anomalies
}

/// Per-ledger event counts used for cross-ledger reconciliation (HM4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerCounts {
    /// Identification events recorded by this service (BVI mirror table).
    pub bvi_identifications: i64,
    /// Match requests observed at the ABIS (biometric matching engine).
    pub abis_match_requests: i64,
    /// Authenticated-voter rollups emitted to the results pipeline.
    pub rollup_authenticated: i64
}

/// Feed freshness for each ledger. Under low/intermittent connectivity these
/// can lag arbitrarily; reconciliation NEVER alarms on stale feeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerMeta {
    pub bvi_age_secs: i64,
    pub abis_age_secs: i64,
    pub rollup_age_secs: i64,
}

/// Outcome of the cross-ledger reconciliation check.
///
/// `Stale` dominates everything: if any feed is older than the max
/// staleness, the verdict is Stale even when counts disagree, because a
/// delta over stale data is not evidence of tampering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossLedgerVerdict {
    /// Any feed older than `max_staleness_secs`; do not draw conclusions.
    Stale { oldest_age_secs: i64 },
    /// All compared ledgers agree within tolerance.
    Consistent,
    /// A pairwise delta exceeds tolerance; names both ledgers and the delta.
    Anomaly { detail: String, severity: Severity },
}

/// Compare ledger counts pairwise against the authenticated-rollup anchor.
///
/// Only two pairs are compared: `bvi_identifications` vs `rollup_authenticated`
/// and `abis_match_requests` vs `rollup_authenticated`. The direct
/// `abis vs bvi` pair is deliberately NOT compared: ABIS match requests are a
/// superset of successful identifications by design — they include retry
/// attempts, no-match verifications and quality-failure re-scans that never
/// become BVI identification events. Their difference is therefore expected
/// to drift with operational conditions, so comparing them directly would
/// produce routine false alarms; each is anchored on the rollup instead.
///
/// Boundary equality (`delta == tolerance`) counts as Consistent.
pub fn cross_ledger_delta(
    counts: &LedgerCounts,
    meta: &LedgerMeta,
    tolerance: i64,
    max_staleness_secs: i64,
) -> CrossLedgerVerdict {
    // Low-connectivity guard: stale inputs are reported as such, never as
    // anomalies.
    let oldest_age_secs = meta
        .bvi_age_secs
        .max(meta.abis_age_secs)
        .max(meta.rollup_age_secs);
    if oldest_age_secs > max_staleness_secs {
        return CrossLedgerVerdict::Stale { oldest_age_secs };
    }

    let pairs = [
        (
            "bvi_identifications",
            counts.bvi_identifications,
            "rollup_authenticated",
            counts.rollup_authenticated,
        ),
        (
            "abis_match_requests",
            counts.abis_match_requests,
            "rollup_authenticated",
            counts.rollup_authenticated,
        ),
    ];

    let mut violations: Vec<String> = Vec::new();
    for (a_name, a_count, b_name, b_count) in pairs {
        let delta = (a_count - b_count).abs();
        if delta > tolerance {
            violations.push(format!(
                "{a_name} ({a_count}) vs {b_name} ({b_count}): delta {delta} exceeds tolerance {tolerance}"
            ));
        }
    }

    if violations.is_empty() {
        return CrossLedgerVerdict::Consistent;
    }
    CrossLedgerVerdict::Anomaly {
        detail: violations.join("; "),
        severity: Severity::High,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(secs: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0)
            .unwrap()
            .checked_add_signed(chrono::Duration::seconds(secs))
            .unwrap()
    }

    // ---- turnover_over_registered ----

    #[test]
    fn fires_when_identified_exceeds_registered() {
        let a = turnover_over_registered("ST-01", 5_100, 5_000).expect("must fire");
        assert_eq!(a.kind, "turnover_over_registered");
        assert_eq!(a.station_code, "ST-01");
        assert_eq!(a.severity, Severity::Critical);
        assert!(a.detail.contains("5100") || a.detail.contains("5,100"));
    }

    #[test]
    fn silent_when_within_roll() {
        assert!(turnover_over_registered("ST-01", 4_999, 5_000).is_none());
        assert!(turnover_over_registered("ST-01", 5_000, 5_000).is_none());
    }

    #[test]
    fn negative_registered_is_malformed_not_anomaly() {
        assert!(turnover_over_registered("ST-01", -3, -5).is_none());
        assert!(turnover_over_registered("ST-01", 0, -1).is_none());
    }

    #[test]
    fn anomaly_serializes_roundtrip() {
        let a = turnover_over_registered("ST-9", 10, 5).unwrap();
        let json = serde_json::to_string(&a).unwrap();
        let back: TurnoutAnomaly = serde_json::from_str(&json).unwrap();
        assert_eq!(back, a);
    }

    // ---- velocity_impossible ----

    fn read(serial: &str, station: &str, at_secs: i64) -> DeviceRead {
        DeviceRead {
            device_serial: serial.to_string(),
            station_code: station.to_string(),
            seen_at: ts(at_secs),
        }
    }

    #[test]
    fn flags_same_device_two_stations_too_fast() {
        let reads = vec![
            read("D1", "ST-A", 0),
            read("D1", "ST-B", 120),
        ];
        let out = velocity_impossible(&reads, 600);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].severity, Severity::High);
        assert!(out[0].detail.contains("ST-A"));
        assert!(out[0].detail.contains("ST-B"));
        assert_eq!(out[0].station_code, "ST-B");
    }

    #[test]
    fn allows_travel_time_above_threshold() {
        let reads = vec![
            read("D1", "ST-A", 0),
            read("D1", "ST-B", 700),
        ];
        assert!(velocity_impossible(&reads, 600).is_empty());
    }

    #[test]
    fn same_station_back_to_back_is_fine() {
        let reads = vec![
            read("D1", "ST-A", 0),
            read("D1", "ST-A", 30),
        ];
        assert!(velocity_impossible(&reads, 600).is_empty());
    }

    #[test]
    fn unsorted_input_is_sorted_by_time() {
        let reads = vec![
            read("D1", "ST-B", 60),
            read("D1", "ST-A", 0),
        ];
        let out = velocity_impossible(&reads, 600);
        assert_eq!(out.len(), 1);
        assert!(out[0].detail.contains("ST-A") && out[0].detail.contains("ST-B"));
    }

    #[test]
    fn devices_are_grouped_independently() {
        let reads = vec![
            read("D1", "ST-A", 0),
            read("D2", "ST-B", 30),
            read("D1", "ST-B", 30),
            read("D2", "ST-C", 90),
        ];
        let out = velocity_impossible(&reads, 600);
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|a| a.kind == "velocity_impossible"));
    }

    #[test]
    fn empty_and_single_read_are_silent() {
        assert!(velocity_impossible(&[], 600).is_empty());
        assert!(velocity_impossible(&[read("D1", "ST-A", 0)], 600).is_empty());
    }

    // ---- burst_zscore ----

    #[test]
    fn too_few_windows_returns_none() {
        assert!(burst_zscore(&[], 0, 10, 3.0).is_none());
        assert!(burst_zscore(&[1.0, 2.0], 0, 10, 3.0).is_none());
    }

    #[test]
    fn zero_stdev_baseline_yields_no_anomaly() {
        let series = [10.0, 10.0, 10.0, 500.0];
        assert!(burst_zscore(&series, 0, 10, 3.0).is_none());
    }

    #[test]
    fn spike_window_fires() {
        let series = [100.0, 105.0, 95.0, 102.0, 400.0];
        let a = burst_zscore(&series, 4, 5, 3.0).expect("spike must fire");
        assert_eq!(a.kind, "burst_zscore");
        assert_eq!(a.severity, Severity::Critical);
        assert!(a.detail.contains('z'));
    }

    #[test]
    fn normal_window_does_not_fire() {
        let series = [100.0, 105.0, 95.0, 102.0, 400.0];
        assert!(burst_zscore(&series, 1, 5, 3.0).is_none());
    }

    #[test]
    fn tested_window_is_excluded_from_baseline() {
        // Baseline over windows != 1 is {100, 100, 100}: stdev 0 -> None.
        let series = [100.0, 999_999.0, 100.0, 100.0];
        assert!(burst_zscore(&series, 1, 4, 3.0).is_none());
    }

    // ---- digit_distribution_chi_square ----

    const UNIFORM: [f64; 10] = [0.1; 10];

    #[test]
    fn uniform_data_gives_high_p_value() {
        let counts: Vec<u64> = vec![98, 101, 103, 99, 100, 97, 102, 100, 104, 96]; // total 1000
        let stats = digit_distribution_chi_square(&counts, &UNIFORM).unwrap();
        assert_eq!(stats.dof, 9.0);
        assert!(stats.p_value > 0.01, "p={} too low for uniform data", stats.p_value);
    }

    #[test]
    fn heavily_skewed_data_gives_tiny_p_value() {
        let counts: Vec<u64> = vec![900, 10, 10, 10, 10, 10, 10, 10, 20, 10];
        let stats = digit_distribution_chi_square(&counts, &UNIFORM).unwrap();
        assert!(stats.p_value < 0.001, "p={} too high for skewed data", stats.p_value);
        assert!(stats.chi_square > 50.0);
    }

    #[test]
    fn perfect_fit_has_zero_statistic() {
        let counts: Vec<u64> = vec![100; 10];
        let stats = digit_distribution_chi_square(&counts, &UNIFORM).unwrap();
        assert!(stats.chi_square.abs() < 1e-12);
        assert!(stats.p_value > 0.99);
    }

    #[test]
    fn length_mismatch_is_error() {
        let err = digit_distribution_chi_square(&[1, 2], &[0.5]).unwrap_err();
        assert!(err.contains("length"));
    }

    #[test]
    fn negative_share_is_error() {
        let err = digit_distribution_chi_square(&[1, 2], &[0.5, -0.1]).unwrap_err();
        assert!(err.contains("non-negative"));
    }

    #[test]
    fn benford_reference_values() {
        // Benford shares; data close to Benford should pass, uniform-ish fail.
        let shares = [
            0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
        ];
        let near_benford = [300_u64, 180, 125, 95, 80, 65, 60, 50, 45];
        let stats = digit_distribution_chi_square(&near_benford, &shares).unwrap();
        assert!(stats.p_value > 0.01);

        let flat_against_benford = [200_u64; 9];
        let stats = digit_distribution_chi_square(&flat_against_benford, &shares).unwrap();
        assert!(stats.p_value < 0.001);
    }

    // ---- evaluate_station ----

    #[test]
    fn combines_both_detectors() {
        let series = [100.0, 105.0, 95.0, 400.0];
        let out = evaluate_station("ST-7", 6_000, 5_000, &series, 3, 4, 3.0);
        assert_eq!(out.len(), 2);
        assert!(out.iter().any(|a| a.kind == "turnover_over_registered"));
        assert!(out.iter().all(|a| a.station_code == "ST-7"));
    }

    #[test]
    fn quiet_station_returns_empty() {
        let series = [100.0, 105.0, 95.0, 400.0];
        let out = evaluate_station("ST-7", 100, 5_000, &series, 0, 4, 3.0);
        assert!(out.is_empty());
    }

    // ---- cross_ledger_delta ----

    fn counts(bvi: i64, abis: i64, rollup: i64) -> LedgerCounts {
        LedgerCounts {
            bvi_identifications: bvi,
            abis_match_requests: abis,
            rollup_authenticated: rollup,
        }
    }

    fn fresh() -> LedgerMeta {
        LedgerMeta { bvi_age_secs: 5, abis_age_secs: 7, rollup_age_secs: 3 }
    }

    #[test]
    fn consistent_within_tolerance() {
        let c = counts(1_000, 1_010, 998);
        assert_eq!(
            cross_ledger_delta(&c, &fresh(), 20, 600),
            CrossLedgerVerdict::Consistent
        );
    }

    #[test]
    fn anomaly_beyond_tolerance_names_ledgers_and_delta() {
        let c = counts(1_000, 1_000, 900);
        match cross_ledger_delta(&c, &fresh(), 50, 600) {
            CrossLedgerVerdict::Anomaly { detail, severity } => {
                assert_eq!(severity, Severity::High);
                assert!(detail.contains("bvi_identifications"), "{detail}");
                assert!(detail.contains("rollup_authenticated"), "{detail}");
                assert!(detail.contains("100"), "{detail}");
            }
            other => panic!("expected Anomaly, got {other:?}"),
        }
    }

    #[test]
    fn stale_dominates_even_when_inconsistent() {
        let meta = LedgerMeta { bvi_age_secs: 30, abis_age_secs: 10, rollup_age_secs: 9_000 };
        let c = counts(1_000, 2_500, 100);
        assert_eq!(
            cross_ledger_delta(&c, &meta, 10, 600),
            CrossLedgerVerdict::Stale { oldest_age_secs: 9_000 }
        );
        // Stale even when counts would otherwise agree.
        let ok = counts(500, 500, 500);
        assert_eq!(
            cross_ledger_delta(&ok, &meta, 10, 600),
            CrossLedgerVerdict::Stale { oldest_age_secs: 9_000 }
        );
    }

    #[test]
    fn boundary_equality_counts_as_consistent() {
        // delta exactly == tolerance on both pairs.
        let c = counts(1_010, 990, 1_000);
        assert_eq!(
            cross_ledger_delta(&c, &fresh(), 10, 600),
            CrossLedgerVerdict::Consistent
        );
        // Staleness boundary exactly == max is still fresh enough.
        let meta = LedgerMeta { bvi_age_secs: 600, abis_age_secs: 600, rollup_age_secs: 600 };
        assert_eq!(
            cross_ledger_delta(&counts(5, 5, 5), &meta, 0, 600),
            CrossLedgerVerdict::Consistent
        );
    }

    #[test]
    fn verdict_serializes_snake_case() {
        let json = serde_json::to_string(&CrossLedgerVerdict::Stale { oldest_age_secs: 7 }).unwrap();
        assert!(json.contains("stale"), "{json}");
        assert!(serde_json::to_string(&CrossLedgerVerdict::Consistent).unwrap().contains("consistent"));
    }
}
