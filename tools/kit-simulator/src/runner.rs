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

use anyhow::Result;
use chrono::Utc;
use rand::{rngs::SmallRng, Rng, SeedableRng as _};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::{load_profile, load_profile_from_path, Profile};
use crate::mock_server::MockServer;
use crate::report::{percentiles, print_run, write_report, EndpointStats, RunReport};
use crate::scenario::{load_scenario, Scenario};

#[derive(Debug)]
struct RequestRecord {
    endpoint: String,
    latency_ms: u64,
    error: bool,
}

/// Entry point for the `run` subcommand.
pub async fn run(
    profile_name: String,
    duration: Duration,
    scenario_path: PathBuf,
    target_url: Option<String>,
    reports_dir: PathBuf,
) -> Result<()> {
    // Support test-mode profile path resolution
    let profile = if let Ok(alt) = std::env::var("EEL_PROFILE_DIR") {
        load_profile_from_path(&crate::config::profile_path(
            &profile_name,
            std::path::Path::new(&alt),
        ))?
    } else {
        load_profile(&profile_name)?
    };

    let scenario = if let Ok(alt) = std::env::var("EEL_SCENARIO_DIR") {
        let p = std::path::Path::new(&alt).join(
            scenario_path
                .file_name()
                .unwrap_or(scenario_path.as_os_str()),
        );
        load_scenario(&p)?
    } else {
        load_scenario(&scenario_path)?
    };

    let test_mode = std::env::var("EEL_KIT_ATTEST_TEST_MODE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let (base_url, _mock) = match target_url {
        Some(url) if !test_mode => (url, None),
        _ => {
            let mock = MockServer::start().await?;
            let url = mock.base_url();
            info!("Dry-run mode: in-process mock at {url}");
            (url, Some(mock))
        }
    };

    let kit_count = profile.kit_count();
    let legacy_count = profile.legacy_kit_count();
    info!(
        "Simulating {kit_count} kits ({legacy_count} legacy Android) for {}s",
        duration.as_secs()
    );

    let started_at = Utc::now();

    // Channel for collecting request records from worker tasks
    let (tx, mut rx) = mpsc::channel::<RequestRecord>(200_000);

    // Spawn workers: group kits into bounded worker pool
    let worker_count = (kit_count / 10).max(1).min(64) as usize;
    let kits_per_worker = (kit_count / worker_count as u64).max(1);
    let legacy_workers = (legacy_count / kits_per_worker.max(1)).max(0) as usize;

    let profile = Arc::new(profile);
    let scenario = Arc::new(scenario);
    let base_url = Arc::new(base_url);

    let mut handles = Vec::with_capacity(worker_count);
    for worker_id in 0..worker_count {
        let tx = tx.clone();
        let profile = Arc::clone(&profile);
        let scenario = Arc::clone(&scenario);
        let base_url = Arc::clone(&base_url);
        let is_legacy = worker_id < legacy_workers;

        handles.push(tokio::spawn(async move {
            run_worker(
                worker_id as u64,
                is_legacy,
                duration,
                profile,
                scenario,
                base_url,
                tx,
            )
            .await
        }));
    }
    drop(tx); // close sender side so recv loop ends when workers finish

    // Aggregate records
    let mut latencies: HashMap<String, Vec<u64>> = HashMap::new();
    let mut errors: HashMap<String, u64> = HashMap::new();

    while let Some(rec) = rx.recv().await {
        latencies
            .entry(rec.endpoint.clone())
            .or_default()
            .push(rec.latency_ms);
        if rec.error {
            *errors.entry(rec.endpoint).or_insert(0) += 1;
        }
    }

    for h in handles {
        h.await??;
    }

    let ended_at = Utc::now();

    let mut endpoint_stats: HashMap<String, EndpointStats> = HashMap::new();
    let mut total_requests = 0u64;
    let mut total_errors = 0u64;

    for (ep, lats) in latencies {
        let count = lats.len() as u64;
        let errs = errors.get(&ep).copied().unwrap_or(0);
        let (p50, p95, p99) = percentiles(lats);
        total_requests += count;
        total_errors += errs;
        endpoint_stats.insert(
            ep,
            EndpointStats {
                count,
                errors: errs,
                p50_ms: p50,
                p95_ms: p95,
                p99_ms: p99,
            },
        );
    }

    let run_id = Uuid::now_v7().to_string();
    let report = RunReport {
        run_id: run_id.clone(),
        profile: profile_name,
        scenario: scenario.name.clone(),
        started_at: started_at.to_rfc3339(),
        ended_at: ended_at.to_rfc3339(),
        kit_count,
        legacy_kit_count: legacy_count,
        total_requests,
        total_errors,
        endpoints: endpoint_stats,
    };

    let path = write_report(&report, &reports_dir)?;
    info!("Report: {:?} (run_id: {run_id})", path);
    print_run(&run_id, &reports_dir)?;

    // Optional Prometheus push (no external dep; plain text format to pushgateway)
    push_metrics_optional(&report).await;

    Ok(())
}

async fn run_worker(
    worker_id: u64,
    is_legacy: bool,
    duration: Duration,
    profile: Arc<Profile>,
    scenario: Arc<Scenario>,
    base_url: Arc<String>,
    tx: mpsc::Sender<RequestRecord>,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // SmallRng is Send: safe to hold across await points in spawned tasks.
    let mut rng = SmallRng::seed_from_u64(worker_id.wrapping_mul(0x9e37_4761).wrapping_add(0x4711));
    let deadline = Instant::now() + duration;
    let sim_start = Instant::now();

    while Instant::now() < deadline {
        let elapsed_min = sim_start.elapsed().as_secs() / 60;
        let phase = scenario.phase_at(elapsed_min);

        let kit_suffix: u32 = rng.gen::<u32>() % 100_000;
        let kit_id = format!("kit-{worker_id}-{kit_suffix}");
        let packet_loss: bool = rng.gen::<f64>() < (phase.telco_profile.packet_loss_pct / 100.0);
        let send_heartbeat: bool = rng.gen::<f64>() < 0.03; // ~1/30s at loop rate

        // BVI identification
        if phase.bvi_rps > 0.0 || profile.bvi_req_per_sec > 0 {
            let path = if is_legacy {
                "/legacy/bvi/identify"
            } else {
                "/bvi/identify"
            };
            if tx
                .send(make_request(&client, &base_url, path, &kit_id).await)
                .await
                .is_err()
            {
                return Ok(());
            }
        }

        // RTS 34A submission
        if phase.rts_concurrent > 0 || profile.rts_concurrent > 0 {
            if tx
                .send(make_request(&client, &base_url, "/rts/submit", &kit_id).await)
                .await
                .is_err()
            {
                return Ok(());
            }
        }

        // Kit heartbeat (30s interval; approximate via probability)
        if send_heartbeat {
            if tx
                .send(make_request(&client, &base_url, "/kit/heartbeat", &kit_id).await)
                .await
                .is_err()
            {
                return Ok(());
            }
        }

        // Telco delay: retransmit penalty on packet loss, otherwise steady pace
        let delay_ms: u64 = if packet_loss {
            let tech_roll: f64 = rng.gen();
            if tech_roll < phase.telco_profile.satellite_fallback_pct / 100.0 {
                rng.gen_range(800u64..=3000) // satellite
            } else {
                rng.gen_range(200u64..=1500) // 3G retransmit
            }
        } else {
            // Pace to bvi_req_per_sec; floor at 50ms to avoid busy-loop
            let target_interval_ms = if profile.bvi_req_per_sec > 0 {
                1000 / profile.bvi_req_per_sec
            } else {
                100
            };
            target_interval_ms.max(50)
        };

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    Ok(())
}

async fn make_request(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    kit_id: &str,
) -> RequestRecord {
    let url = format!("{base_url}{path}");
    let t0 = Instant::now();
    let result = client
        .post(&url)
        .header("X-Kit-ID", kit_id)
        .header("X-Attestation-Token", attestation_token(kit_id))
        .json(&serde_json::json!({
            "kit_id": kit_id,
            "ts": chrono::Utc::now().timestamp_millis()
        }))
        .send()
        .await;
    let latency_ms = t0.elapsed().as_millis() as u64;
    match result {
        Ok(resp) => RequestRecord {
            endpoint: path_to_name(path),
            latency_ms,
            error: !resp.status().is_success(),
        },
        Err(_) => RequestRecord {
            endpoint: path_to_name(path),
            latency_ms,
            error: true,
        },
    }
}

/// Stub attestation token accepted when EEL_KIT_ATTEST_TEST_MODE=true.
fn attestation_token(kit_id: &str) -> String {
    // hex-encode the kit_id bytes as the "payload" to make the token look non-trivial
    let payload: String = kit_id.bytes().map(|b| format!("{b:02x}")).collect();
    format!("test-attest.{payload}.stub")
}

fn path_to_name(path: &str) -> String {
    path.trim_start_matches('/').replace('/', "-")
}

async fn push_metrics_optional(report: &RunReport) {
    let Ok(gw) = std::env::var("EEL_PROMETHEUS_PUSHGATEWAY") else {
        return;
    };
    let mut body = String::new();
    for (ep, stats) in &report.endpoints {
        let safe = ep.replace('-', "_");
        body.push_str(&format!(
            "kit_simulator_p99_ms{{endpoint=\"{safe}\",profile=\"{}\",run_id=\"{}\"}} {}\n",
            report.profile, report.run_id, stats.p99_ms
        ));
    }
    let url = format!(
        "{}/metrics/job/kit-simulator/instance/{}",
        gw.trim_end_matches('/'),
        report.run_id
    );
    match reqwest::Client::new().post(&url).body(body).send().await {
        Ok(_) => info!("Metrics pushed to {gw}"),
        Err(e) => warn!("Prometheus push failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_name_strips_and_replaces() {
        assert_eq!(path_to_name("/bvi/identify"), "bvi-identify");
        assert_eq!(path_to_name("/rts/submit"), "rts-submit");
        assert_eq!(path_to_name("/kit/heartbeat"), "kit-heartbeat");
    }

    #[test]
    fn attestation_token_format() {
        let t = attestation_token("kit-007");
        assert!(t.starts_with("test-attest."), "wrong prefix: {t}");
        assert!(t.ends_with(".stub"), "wrong suffix: {t}");
        // Payload is non-empty
        let mid = t
            .trim_start_matches("test-attest.")
            .trim_end_matches(".stub");
        assert!(!mid.is_empty());
    }

    #[tokio::test]
    async fn mock_server_responds_200() {
        let mock = MockServer::start().await.unwrap();
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/bvi/identify", mock.base_url()))
            .json(&serde_json::json!({"kit_id": "test"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        mock.stop();
    }

    #[tokio::test]
    async fn dry_run_produces_report() {
        let dir = std::path::PathBuf::from(format!("/tmp/kit-sim-dr-{}", std::process::id()));

        // Point the runner at the workspace-root scenario and profiles via env
        let ws = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();

        std::env::set_var("EEL_KIT_ATTEST_TEST_MODE", "true");
        std::env::set_var(
            "EEL_PROFILE_DIR",
            ws.join("loadtest/profiles").to_str().unwrap(),
        );
        std::env::set_var(
            "EEL_SCENARIO_DIR",
            ws.join("loadtest/scenarios").to_str().unwrap(),
        );

        run(
            "local".into(),
            Duration::from_secs(3),
            std::path::PathBuf::from("election-day.yaml"), // resolved via EEL_SCENARIO_DIR
            None,
            dir.clone(),
        )
        .await
        .unwrap();

        let entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
            .collect();
        assert!(
            !entries.is_empty(),
            "expected at least one report JSON in {:?}",
            dir
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
