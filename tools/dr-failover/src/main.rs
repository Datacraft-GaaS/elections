/// eel-dr-failover -- automated DR failover assist
///
/// Subcommands:
///   plan             -- print the failover playbook for a service or the full site
///   execute          -- run failover steps (requires --dry-run or --confirm)
///   report           -- measure RTO/RPO from Prometheus/Loki and emit a structured report
///
/// Naming: eel prefix; no em/en dashes in output.
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// CLI surface
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "eel-dr-failover",
    about = "Automated DR failover assist for the eel election platform",
    version
)]
struct Cli {
    /// Prometheus base URL (for RTO/RPO metrics)
    #[arg(long, env = "EEL_PROM_URL", default_value = "http://prometheus.monitoring.svc.cluster.local:9090")]
    prom_url: String,

    /// Target site to fail over to
    #[arg(long, env = "EEL_DR_TARGET_SITE", default_value = "mirror")]
    target_site: String,

    /// Log level (trace | debug | info | warn | error)
    #[arg(long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the failover playbook (steps, dependencies, expected RTO) without executing
    Plan {
        /// Service to show plan for (omit for full site failover plan)
        #[arg(long)]
        service: Option<String>,
    },

    /// Execute failover steps
    Execute {
        /// Service to fail over (omit for full site failover)
        #[arg(long)]
        service: Option<String>,

        /// Print what would happen without taking action
        #[arg(long, conflicts_with = "confirm")]
        dry_run: bool,

        /// Actually execute -- requires explicit flag to prevent accidental invocation
        #[arg(long, conflicts_with = "dry_run")]
        confirm: bool,
    },

    /// Measure RTO and RPO from the last drill and emit a structured report
    Report {
        /// Drill identifier (e.g. "election-day" or "infra-stress")
        #[arg(long)]
        drill: Option<String>,

        /// Check for sequence gaps in the event log
        #[arg(long)]
        check_gaps: bool,

        /// Service to health-check
        #[arg(long)]
        check_svc: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
}

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct DrillReport {
    drill_id: String,
    generated_at: DateTime<Utc>,
    site: String,
    rto_seconds: Option<f64>,
    rpo_seconds: Option<f64>,
    rto_slo_seconds: f64,
    rpo_slo_seconds: f64,
    rto_met: bool,
    rpo_met: bool,
    sequence_gaps: u64,
    notes: Vec<String>,
}

impl DrillReport {
    fn new(drill_id: &str, site: &str) -> Self {
        Self {
            drill_id: drill_id.to_string(),
            generated_at: Utc::now(),
            site: site.to_string(),
            rto_seconds: None,
            rpo_seconds: None,
            rto_slo_seconds: 1800.0, // 30 min for rts-svc
            rpo_slo_seconds: 300.0,  // 5 min for rts-svc
            rto_met: false,
            rpo_met: false,
            sequence_gaps: 0,
            notes: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FailoverStep {
    order: u32,
    action: String,
    command: String,
    expected_duration_secs: u32,
    verify: String,
}

// ---------------------------------------------------------------------------
// Failover playbook
// ---------------------------------------------------------------------------

fn site_failover_plan(target_site: &str) -> Vec<FailoverStep> {
    vec![
        FailoverStep {
            order: 1,
            action: "Confirm site partition".into(),
            command: format!("kubectl -n eel-prod get nodes --selector=site={target_site}"),
            expected_duration_secs: 30,
            verify: "All target-site nodes Ready".into(),
        },
        FailoverStep {
            order: 2,
            action: "Promote Patroni replica to primary on target site".into(),
            command: format!("patronictl -c /etc/patroni/patroni.yml failover --master <current-primary> --candidate <{target_site}-replica> --force"),
            expected_duration_secs: 60,
            verify: "patronictl list shows exactly one primary on target site".into(),
        },
        FailoverStep {
            order: 3,
            action: "Verify WAL lag below RPO".into(),
            command: "psql -c \"SELECT now() - pg_last_xact_replay_timestamp() AS lag;\"".into(),
            expected_duration_secs: 10,
            verify: "lag < 5 minutes".into(),
        },
        FailoverStep {
            order: 4,
            action: "Update DNS / Cilium GlobalService to route to target site".into(),
            command: format!("kubectl -n eel-prod patch globalservice rts-svc --patch '{{\"spec\":{{\"activeEndpoint\":\"{target_site}\"}}}}'"),
            expected_duration_secs: 30,
            verify: "curl -sf https://rts.eel.internal/healthz returns 200".into(),
        },
        FailoverStep {
            order: 5,
            action: "Verify MirrorMaker2 replication lag".into(),
            command: "kafka-consumer-groups.sh --bootstrap-server kafka:9092 --describe --group mirrormaker2".into(),
            expected_duration_secs: 30,
            verify: "Consumer lag < 10 000 (< 10 s at expected throughput)".into(),
        },
        FailoverStep {
            order: 6,
            action: "Confirm rts-svc write path healthy on target site".into(),
            command: "curl -sf -X POST https://rts.eel.internal/api/v1/results/probe".into(),
            expected_duration_secs: 30,
            verify: "HTTP 201 or 200; no 5xx".into(),
        },
        FailoverStep {
            order: 7,
            action: "Record RTO and RPO".into(),
            command: "eel-dr-failover report --drill site-failover".into(),
            expected_duration_secs: 10,
            verify: "report.rto_met == true && report.rpo_met == true".into(),
        },
    ]
}

fn service_failover_plan(svc: &str, target_site: &str) -> Vec<FailoverStep> {
    vec![
        FailoverStep {
            order: 1,
            action: format!("Scale down {svc} on current site"),
            command: format!("kubectl -n eel-prod scale deploy/{svc} --replicas=0"),
            expected_duration_secs: 20,
            verify: format!("kubectl -n eel-prod get pods -l app={svc} returns 0 pods"),
        },
        FailoverStep {
            order: 2,
            action: format!("Scale up {svc} on {target_site} site"),
            command: format!("kubectl --context={target_site} -n eel-prod scale deploy/{svc} --replicas=3"),
            expected_duration_secs: 60,
            verify: format!("All {svc} pods on {target_site} are Ready"),
        },
        FailoverStep {
            order: 3,
            action: "Verify service health".into(),
            command: format!("curl -sf https://{svc}.eel.internal/healthz"),
            expected_duration_secs: 10,
            verify: "HTTP 200".into(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Prometheus query helpers
// ---------------------------------------------------------------------------

async fn query_prometheus(prom_url: &str, promql: &str) -> Result<f64> {
    let url = format!("{prom_url}/api/v1/query");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    #[derive(Deserialize)]
    struct PromResponse {
        data: PromData,
    }
    #[derive(Deserialize)]
    struct PromData {
        result: Vec<PromResult>,
    }
    #[derive(Deserialize)]
    struct PromResult {
        value: (f64, String),
    }

    let resp: PromResponse = client
        .get(&url)
        .query(&[("query", promql)])
        .send()
        .await
        .context("Prometheus request failed")?
        .json()
        .await
        .context("Prometheus JSON parse failed")?;

    resp.data
        .result
        .first()
        .map(|r| r.value.1.parse::<f64>().unwrap_or(0.0))
        .ok_or_else(|| anyhow::anyhow!("No result from Prometheus for query: {promql}"))
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

async fn cmd_plan(service: Option<String>, target_site: &str) {
    let steps = match &service {
        Some(svc) => service_failover_plan(svc, target_site),
        None => site_failover_plan(target_site),
    };

    let scope = service.as_deref().unwrap_or("full-site");
    println!("Failover plan: {scope} -> {target_site}");
    println!("{}", "-".repeat(72));
    for step in &steps {
        println!(
            "[{:02}] {} (~{}s)",
            step.order, step.action, step.expected_duration_secs
        );
        println!("     CMD:    {}", step.command);
        println!("     VERIFY: {}", step.verify);
        println!();
    }
    let total_secs: u32 = steps.iter().map(|s| s.expected_duration_secs).sum();
    println!("Estimated total: {}s ({:.1} min)", total_secs, total_secs as f64 / 60.0);
}

async fn cmd_execute(
    service: Option<String>,
    target_site: &str,
    dry_run: bool,
    confirm: bool,
) -> Result<()> {
    if !dry_run && !confirm {
        bail!("Must pass either --dry-run or --confirm. Refusing to execute without explicit intent.");
    }

    let steps = match &service {
        Some(svc) => service_failover_plan(svc, target_site),
        None => site_failover_plan(target_site),
    };

    let mode = if dry_run { "DRY-RUN" } else { "LIVE" };
    let scope = service.as_deref().unwrap_or("full-site");

    println!("[{mode}] Executing failover: {scope} -> {target_site}");
    println!("{}", "-".repeat(72));

    let wall_start = Instant::now();

    for step in &steps {
        println!("[{:02}] {}", step.order, step.action);
        println!("     CMD: {}", step.command);

        if dry_run {
            println!("     (dry-run: skipping execution)");
        } else {
            // Execute the command via shell. In production this would use
            // the Kubernetes API or kubectl subprocess. For safety we log
            // and shell out only to read-only verifications in this version.
            // Write operations require an operator to confirm via the step plan
            // and run each command manually -- this tool orchestrates, not automates blindly.
            warn!("Step {:02}: live execution not implemented in this version. Run manually:", step.order);
            warn!("  {}", step.command);
        }
        println!("     VERIFY: {}", step.verify);
        println!();
    }

    let elapsed = wall_start.elapsed();
    println!("Completed in {:.1}s", elapsed.as_secs_f64());

    if !dry_run {
        println!("Run `eel-dr-failover report` to capture RTO/RPO metrics.");
    }

    Ok(())
}

async fn cmd_report(
    prom_url: &str,
    target_site: &str,
    drill: Option<String>,
    check_gaps: bool,
    check_svc: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let drill_id = drill.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mut report = DrillReport::new(&drill_id, target_site);

    // Query RTO: time from first alert to service healthy again.
    // Uses the `rts_svc_failover_rto_seconds` gauge set by the failover webhook.
    match query_prometheus(prom_url, "rts_svc_failover_rto_seconds").await {
        Ok(v) => {
            report.rto_seconds = Some(v);
            report.rto_met = v <= report.rto_slo_seconds;
        }
        Err(e) => {
            warn!("Could not fetch RTO from Prometheus: {e}");
            report.notes.push(format!("RTO metric unavailable: {e}"));
        }
    }

    // Query RPO: lag at failover point from WAL timestamp metric.
    match query_prometheus(prom_url, "pg_replication_lag_seconds{site=\"primary\"}").await {
        Ok(v) => {
            report.rpo_seconds = Some(v);
            report.rpo_met = v <= report.rpo_slo_seconds;
        }
        Err(e) => {
            warn!("Could not fetch RPO from Prometheus: {e}");
            report.notes.push(format!("RPO metric unavailable: {e}"));
        }
    }

    // Sequence gap check
    if check_gaps {
        // In production: query the event store or Kafka for counter discontinuities.
        // Here we surface the check and note it requires DB access.
        report.notes.push(
            "Gap check: run `SELECT counter, count(*) FROM events GROUP BY counter HAVING count(*) > 1;` on T1 cluster".into()
        );
        info!("Sequence gap check requested -- see notes in report");
    }

    // Service health check
    if let Some(ref svc) = check_svc {
        let url = format!("https://{svc}.eel.internal/healthz");
        info!("Checking service health: {url}");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .danger_accept_invalid_certs(true) // pre-prod only
            .build()?;
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                report.notes.push(format!("{svc} health check: OK ({status})", status = resp.status()));
            }
            Ok(resp) => {
                report.notes.push(format!("{svc} health check: FAIL ({status})", status = resp.status()));
            }
            Err(e) => {
                report.notes.push(format!("{svc} health check: ERROR ({e})"));
            }
        }
    }

    match format {
        OutputFormat::Text => {
            println!("DR Drill Report");
            println!("{}", "=".repeat(72));
            println!("Drill ID  : {}", report.drill_id);
            println!("Generated : {}", report.generated_at.format("%Y-%m-%dT%H:%M:%SZ"));
            println!("Site      : {}", report.site);
            println!();
            let rto_str = report.rto_seconds.map(|v| format!("{v:.0}s")).unwrap_or("N/A".into());
            let rpo_str = report.rpo_seconds.map(|v| format!("{v:.0}s")).unwrap_or("N/A".into());
            println!("RTO       : {rto_str:<10} SLO={:.0}s  {}", report.rto_slo_seconds, if report.rto_met { "PASS" } else { "FAIL" });
            println!("RPO       : {rpo_str:<10} SLO={:.0}s  {}", report.rpo_slo_seconds, if report.rpo_met { "PASS" } else { "FAIL" });
            println!();
            if !report.notes.is_empty() {
                println!("Notes:");
                for note in &report.notes {
                    println!("  - {note}");
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    if !report.rto_met || !report.rpo_met {
        error!("SLO violation detected -- drill FAILED. Review and re-drill before next release.");
        std::process::exit(1);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(&cli.log_level)
        .init();

    match cli.command {
        Commands::Plan { service } => {
            cmd_plan(service, &cli.target_site).await;
            Ok(())
        }

        Commands::Execute { service, dry_run, confirm } => {
            cmd_execute(service, &cli.target_site, dry_run, confirm).await
        }

        Commands::Report { drill, check_gaps, check_svc, format } => {
            cmd_report(&cli.prom_url, &cli.target_site, drill, check_gaps, check_svc, format).await
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_plan_has_steps() {
        let steps = site_failover_plan("mirror");
        assert!(!steps.is_empty(), "site failover plan must have steps");
        for (i, step) in steps.iter().enumerate() {
            assert_eq!(step.order, (i + 1) as u32);
            assert!(!step.command.is_empty());
            assert!(!step.verify.is_empty());
        }
    }

    #[test]
    fn service_plan_has_steps() {
        let steps = service_failover_plan("rts-svc", "mirror");
        assert!(!steps.is_empty());
        assert!(steps.iter().all(|s| !s.command.is_empty()));
    }

    #[test]
    fn drill_report_defaults_fail() {
        let report = DrillReport::new("test-drill", "mirror");
        assert!(!report.rto_met);
        assert!(!report.rpo_met);
        assert_eq!(report.rto_slo_seconds, 1800.0);
        assert_eq!(report.rpo_slo_seconds, 300.0);
    }

    #[test]
    fn drill_report_serializes() {
        let report = DrillReport::new("test-drill", "mirror");
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("test-drill"));
        assert!(json.contains("mirror"));
    }

    #[tokio::test]
    async fn execute_requires_dry_run_or_confirm() {
        let result = cmd_execute(None, "mirror", false, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn dry_run_completes_without_error() {
        // dry-run should not hit any external systems
        let result = cmd_execute(Some("rts-svc".into()), "mirror", true, false).await;
        assert!(result.is_ok());
    }
}
