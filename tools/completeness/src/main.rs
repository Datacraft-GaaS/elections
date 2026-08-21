use eel_completeness_lib::{allowlist, contract_closure, report, scan, trace};

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "eel-completeness",
    about = "EEL completeness scanner — finds stubs, maps test coverage, verifies contract closure",
    version
)]
struct Cli {
    /// Repository root to scan (default: current directory)
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    /// Allowlist file path (default: .completeness-allowlist.yaml at repo root)
    #[arg(long)]
    allowlist: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Walk the repo and emit stub/skip/todo findings
    Scan {
        /// Fail if any new (non-allowlisted) findings appear (PR gate)
        #[arg(long)]
        fail_on_new: bool,

        /// Fail on any finding including allowlisted ones (main branch gate)
        #[arg(long)]
        strict: bool,

        /// Output format: human | json | junit
        #[arg(long, default_value = "human")]
        format: String,
    },

    /// Build a traceability coverage map from test trace annotations
    Trace {
        /// Path to TRACEABILITY.yaml
        #[arg(long, default_value = "docs/plans/TRACEABILITY.yaml")]
        traceability: PathBuf,

        /// Fail if any EC-ID row has zero passing tests
        #[arg(long)]
        fail_on_gap: bool,
    },

    /// Emit docs/audit/completeness-report.md combining scan + trace
    Report,

    /// Verify contract closure: every service pair, path, and event has coverage
    ContractClosure {
        /// Fail on missing coverage
        #[arg(long)]
        fail: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let root = cli.root.canonicalize().unwrap_or(cli.root.clone());
    let allowlist_path = cli
        .allowlist
        .unwrap_or_else(|| root.join(".completeness-allowlist.yaml"));

    let al = allowlist::load(&allowlist_path)?;

    match &cli.command {
        Commands::Scan {
            fail_on_new,
            strict,
            format,
        } => {
            let findings = scan::run(&root, &al)?;
            let exit_code = scan::emit(&findings, format, *fail_on_new, *strict)?;
            std::process::exit(exit_code);
        }
        Commands::Trace {
            traceability,
            fail_on_gap,
        } => {
            let exit_code = trace::run(&root, traceability, *fail_on_gap)?;
            std::process::exit(exit_code);
        }
        Commands::Report => {
            report::run(&root, &al)?;
        }
        Commands::ContractClosure { fail } => {
            let exit_code = contract_closure::run(&root, *fail)?;
            std::process::exit(exit_code);
        }
    }

    Ok(())
}
