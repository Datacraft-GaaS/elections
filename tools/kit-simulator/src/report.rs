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

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Per-endpoint latency summary written into each run report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub count: u64,
    pub errors: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

/// Top-level report written to loadtest/reports/run-{run_id}.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub run_id: String,
    pub profile: String,
    pub scenario: String,
    pub started_at: String,
    pub ended_at: String,
    pub kit_count: u64,
    pub legacy_kit_count: u64,
    pub total_requests: u64,
    pub total_errors: u64,
    pub endpoints: HashMap<String, EndpointStats>,
}

impl RunReport {
    pub fn report_path(run_id: &str, reports_dir: &Path) -> PathBuf {
        reports_dir.join(format!("run-{}.json", run_id))
    }
}

/// Compute (p50, p95, p99) from a vec of latencies in ms.
/// The vec is sorted in place.
pub fn percentiles(mut values: Vec<u64>) -> (u64, u64, u64) {
    if values.is_empty() {
        return (0, 0, 0);
    }
    values.sort_unstable();
    let n = values.len();
    // ceil(n*p/100) - 1 in integer arithmetic = (n*p - 1) / 100, clamped to valid range.
    let idx = |p: usize| (n * p).saturating_sub(1) / 100;
    let p50 = values[idx(50)];
    let p95 = values[idx(95).min(n - 1)];
    let p99 = values[idx(99).min(n - 1)];
    (p50, p95, p99)
}

pub fn write_report(report: &RunReport, reports_dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(reports_dir)?;
    let path = RunReport::report_path(&report.run_id, reports_dir);
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(&path, json).with_context(|| format!("writing report to {:?}", path))?;
    Ok(path)
}

pub fn read_report(run_id: &str, reports_dir: &Path) -> Result<RunReport> {
    let path = RunReport::report_path(run_id, reports_dir);
    let f = std::fs::File::open(&path).with_context(|| format!("opening report {:?}", path))?;
    serde_json::from_reader(f).with_context(|| format!("parsing report {:?}", path))
}

pub fn print_run(run_id: &str, reports_dir: &Path) -> Result<()> {
    let r = read_report(run_id, reports_dir)?;
    println!();
    println!("Run       : {}", r.run_id);
    println!("Profile   : {}  Scenario: {}", r.profile, r.scenario);
    println!("Started   : {}  Ended: {}", r.started_at, r.ended_at);
    println!(
        "Kits      : {} ({} legacy Android)",
        r.kit_count, r.legacy_kit_count
    );
    println!(
        "Requests  : {}  Errors: {} ({:.1}%)",
        r.total_requests,
        r.total_errors,
        if r.total_requests == 0 {
            0.0
        } else {
            r.total_errors as f64 / r.total_requests as f64 * 100.0
        }
    );
    println!();
    println!(
        "{:<35} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "endpoint", "count", "errors", "p50ms", "p95ms", "p99ms"
    );
    println!("{}", "-".repeat(80));
    let mut eps: Vec<_> = r.endpoints.iter().collect();
    eps.sort_by_key(|(k, _)| k.as_str());
    for (ep, s) in eps {
        println!(
            "{:<35} {:>8} {:>8} {:>8} {:>8} {:>8}",
            ep, s.count, s.errors, s.p50_ms, s.p95_ms, s.p99_ms
        );
    }
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_sequential_100() {
        let v: Vec<u64> = (1..=100).collect();
        let (p50, p95, p99) = percentiles(v);
        assert_eq!(p50, 50);
        assert_eq!(p95, 95);
        assert_eq!(p99, 99);
    }

    #[test]
    fn percentiles_single_element() {
        let (p50, p95, p99) = percentiles(vec![42]);
        assert_eq!((p50, p95, p99), (42, 42, 42));
    }

    #[test]
    fn percentiles_empty_returns_zeros() {
        let (p50, p95, p99) = percentiles(vec![]);
        assert_eq!((p50, p95, p99), (0, 0, 0));
    }

    #[test]
    fn percentiles_two_elements() {
        let (p50, p95, p99) = percentiles(vec![10, 100]);
        assert_eq!(p50, 10);
        // (2 * 95 / 100).min(1) = 1
        assert_eq!(p95, 100);
        assert_eq!(p99, 100);
    }

    fn sample_report(run_id: &str, p99: u64) -> RunReport {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "bvi-identify".into(),
            EndpointStats {
                count: 200,
                errors: 2,
                p50_ms: 20,
                p95_ms: 45,
                p99_ms: p99,
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
            total_errors: 2,
            endpoints,
        }
    }

    #[test]
    fn write_read_roundtrip() {
        let dir = std::path::Path::new("/tmp/kit-sim-report-test");
        let report = sample_report("rr-001", 80);
        let path = write_report(&report, dir).unwrap();
        let loaded = read_report("rr-001", dir).unwrap();
        assert_eq!(loaded.run_id, "rr-001");
        assert_eq!(loaded.endpoints["bvi-identify"].p99_ms, 80);
        std::fs::remove_file(path).ok();
        std::fs::remove_dir(dir).ok();
    }
}
