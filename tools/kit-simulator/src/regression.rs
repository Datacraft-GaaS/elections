use anyhow::{bail, Result};
use std::path::Path;

use crate::report::{read_report, EndpointStats, RunReport};

/// Maximum allowed p99 increase before the gate fires.
const THRESHOLD: f64 = 0.10;

pub(crate) struct Row {
    endpoint: String,
    base_p99: u64,
    cand_p99: u64,
    delta_pct: f64,
    regressed: bool,
}

pub fn compare(baseline_id: &str, candidate_id: &str, reports_dir: &Path) -> Result<()> {
    let baseline = read_report(baseline_id, reports_dir)?;
    let candidate = read_report(candidate_id, reports_dir)?;
    let rows = build_rows(&baseline, &candidate);

    println!("Regression gate: baseline={baseline_id}  candidate={candidate_id}");
    println!(
        "{:<35} {:>10} {:>12} {:>10}  status",
        "endpoint", "base_p99", "cand_p99", "delta%"
    );
    println!("{}", "-".repeat(80));

    for r in &rows {
        println!(
            "{:<35} {:>10} {:>12} {:>9.1}%  {}",
            r.endpoint,
            r.base_p99,
            r.cand_p99,
            r.delta_pct * 100.0,
            if r.regressed { "FAIL" } else { "ok" }
        );
    }

    let failures: Vec<_> = rows.iter().filter(|r| r.regressed).collect();
    if !failures.is_empty() {
        bail!(
            "{} endpoint(s) regressed >10% at p99: {}",
            failures.len(),
            failures
                .iter()
                .map(|r| r.endpoint.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!(
        "All endpoints within regression gate (threshold +{:.0}%).",
        THRESHOLD * 100.0
    );
    Ok(())
}

/// Build sorted delta rows from two reports.
pub fn build_rows(baseline: &RunReport, candidate: &RunReport) -> Vec<Row> {
    let mut rows: Vec<Row> = baseline
        .endpoints
        .iter()
        .filter_map(|(ep, base)| {
            candidate
                .endpoints
                .get(ep)
                .map(|cand| delta_row(ep, base, cand))
        })
        .collect();
    rows.sort_by(|a, b| {
        b.delta_pct
            .partial_cmp(&a.delta_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows
}

fn delta_row(endpoint: &str, base: &EndpointStats, cand: &EndpointStats) -> Row {
    let delta_pct = if base.p99_ms == 0 {
        0.0
    } else {
        (cand.p99_ms as f64 - base.p99_ms as f64) / base.p99_ms as f64
    };
    Row {
        endpoint: endpoint.to_string(),
        base_p99: base.p99_ms,
        cand_p99: cand.p99_ms,
        delta_pct,
        regressed: delta_pct > THRESHOLD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{write_report, EndpointStats, RunReport};
    use std::collections::HashMap;

    fn report(run_id: &str, bvi_p99: u64, rts_p99: u64) -> RunReport {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "bvi-identify".into(),
            EndpointStats {
                count: 100,
                errors: 0,
                p50_ms: 20,
                p95_ms: 40,
                p99_ms: bvi_p99,
            },
        );
        endpoints.insert(
            "rts-submit".into(),
            EndpointStats {
                count: 100,
                errors: 0,
                p50_ms: 30,
                p95_ms: 60,
                p99_ms: rts_p99,
            },
        );
        RunReport {
            run_id: run_id.into(),
            profile: "local".into(),
            scenario: "election-day".into(),
            started_at: "2026-08-21T06:00:00Z".into(),
            ended_at: "2026-08-21T06:01:00Z".into(),
            kit_count: 650,
            legacy_kit_count: 143,
            total_requests: 200,
            total_errors: 0,
            endpoints,
        }
    }

    #[test]
    fn passes_within_threshold() {
        let dir = Path::new("/tmp/kit-sim-reg-pass");
        let base = report("reg-base-1", 100, 200);
        let cand = report("reg-cand-1", 109, 218); // 9% / 9%: ok
        write_report(&base, dir).unwrap();
        write_report(&cand, dir).unwrap();
        compare("reg-base-1", "reg-cand-1", dir).unwrap();
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn fails_above_threshold() {
        let dir = Path::new("/tmp/kit-sim-reg-fail");
        let base = report("reg-base-2", 100, 200);
        let cand = report("reg-cand-2", 115, 200); // bvi 15%: fail
        write_report(&base, dir).unwrap();
        write_report(&cand, dir).unwrap();
        assert!(compare("reg-base-2", "reg-cand-2", dir).is_err());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn exactly_at_threshold_passes() {
        let dir = Path::new("/tmp/kit-sim-reg-exact");
        let base = report("reg-base-3", 100, 200);
        // 10% exactly is NOT > threshold, should pass
        let cand = report("reg-cand-3", 110, 220);
        write_report(&base, dir).unwrap();
        write_report(&cand, dir).unwrap();
        compare("reg-base-3", "reg-cand-3", dir).unwrap();
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn delta_calculation_correct() {
        let base_stats = EndpointStats {
            count: 100,
            errors: 0,
            p50_ms: 20,
            p95_ms: 40,
            p99_ms: 200,
        };
        let cand_stats = EndpointStats {
            count: 100,
            errors: 0,
            p50_ms: 20,
            p95_ms: 40,
            p99_ms: 240,
        };
        let row = delta_row("ep", &base_stats, &cand_stats);
        assert!((row.delta_pct - 0.2).abs() < 1e-9);
        assert!(row.regressed);
    }

    #[test]
    fn delta_zero_base_does_not_panic() {
        let base_stats = EndpointStats {
            count: 0,
            errors: 0,
            p50_ms: 0,
            p95_ms: 0,
            p99_ms: 0,
        };
        let cand_stats = EndpointStats {
            count: 10,
            errors: 0,
            p50_ms: 5,
            p95_ms: 10,
            p99_ms: 50,
        };
        let row = delta_row("ep", &base_stats, &cand_stats);
        assert!((row.delta_pct).abs() < 1e-9);
        assert!(!row.regressed);
    }
}
