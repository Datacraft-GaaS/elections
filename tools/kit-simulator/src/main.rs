mod config;
mod mock_server;
mod regression;
mod report;
mod runner;
mod scenario;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;

/// KIEMS election-day load simulator.
/// Simulates 65 000 virtual KIEMS kits (BVI 1:1 identification + RTS 34A submission)
/// over a configurable timeline with telco impairment profiles and a p99 regression gate.
#[derive(Parser)]
#[command(name = "kit-simulator", version)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Simulate election-day traffic against a target cluster (or built-in mock).
	Run {
		/// Load-test profile name: local | staging | pre-prod | prod-dry-run.
		#[arg(long, default_value = "local")]
		profile: String,

		/// How long to run. Suffix: s (seconds), m (minutes), h (hours).
		#[arg(long, default_value = "60s")]
		duration: String,

		/// Path to the scenario YAML (e.g. loadtest/scenarios/election-day.yaml).
		#[arg(long)]
		scenario: PathBuf,

		/// Base URL of target services. Omit to use the built-in in-process mock server.
		#[arg(long)]
		target_url: Option<String>,

		/// Directory where run reports are written.
		#[arg(long, default_value = "loadtest/reports")]
		reports_dir: PathBuf,
	},

	/// Compare two run reports and enforce the p99 regression gate (10% threshold).
	Compare {
		/// Run-id of the baseline report.
		#[arg(long)]
		baseline: String,

		/// Run-id of the candidate report.
		#[arg(long)]
		candidate: String,

		/// Directory containing the report files.
		#[arg(long, default_value = "loadtest/reports")]
		reports_dir: PathBuf,
	},

	/// Pretty-print a stored run report.
	Report {
		/// Run-id to display.
		#[arg(long, value_name = "ID")]
		run: String,

		#[arg(long, default_value = "loadtest/reports")]
		reports_dir: PathBuf,
	},
}

fn parse_duration(s: &str) -> Result<Duration> {
	if let Some(v) = s.strip_suffix('s') {
		Ok(Duration::from_secs(v.parse()?))
	} else if let Some(v) = s.strip_suffix('m') {
		Ok(Duration::from_secs(v.parse::<u64>()? * 60))
	} else if let Some(v) = s.strip_suffix('h') {
		Ok(Duration::from_secs(v.parse::<u64>()? * 3600))
	} else {
		bail!("invalid duration '{}': use suffix s / m / h", s)
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
		)
		.init();

	let cli = Cli::parse();

	match cli.command {
		Command::Run { profile, duration, scenario, target_url, reports_dir } => {
			let dur = parse_duration(&duration)?;
			runner::run(profile, dur, scenario, target_url, reports_dir).await?;
		}
		Command::Compare { baseline, candidate, reports_dir } => {
			regression::compare(&baseline, &candidate, &reports_dir)?;
		}
		Command::Report { run, reports_dir } => {
			report::print_run(&run, &reports_dir)?;
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_duration_seconds() {
		assert_eq!(parse_duration("60s").unwrap(), Duration::from_secs(60));
	}

	#[test]
	fn parse_duration_minutes() {
		assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
	}

	#[test]
	fn parse_duration_hours() {
		assert_eq!(parse_duration("12h").unwrap(), Duration::from_secs(43200));
	}

	#[test]
	fn parse_duration_invalid_suffix_errors() {
		assert!(parse_duration("1d").is_err());
		assert!(parse_duration("10").is_err());
		assert!(parse_duration("").is_err());
	}
}
