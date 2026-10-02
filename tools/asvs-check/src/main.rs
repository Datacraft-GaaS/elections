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

use eel_asvs_check_lib::{manifest, report, scan};

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "eel-asvs-check",
    about = "Maps ASVS L2/L3 requirements to test trace tags and reports coverage gaps",
    version
)]
struct Cli {
    /// Repository root to scan (default: current directory)
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    /// Path to ASVS manifest YAML (default: tools/asvs-check/asvs-manifest.yaml under root)
    #[arg(long)]
    manifest: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Walk the repo, collect test trace tags, and print coverage per ASVS requirement
    Scan,

    /// Emit a compliance report
    Report {
        /// Output format: html | md | json
        #[arg(long, default_value = "md")]
        format: String,

        /// Write output to file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Exit non-zero if any L3 (or L2 with --all) requirement has no test coverage
    Check {
        /// Also fail on L2 gaps, not just L3
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let root = cli.root.canonicalize().unwrap_or(cli.root.clone());
    let manifest_path = cli
        .manifest
        .unwrap_or_else(|| manifest::default_manifest_path(&root));

    let requirements = manifest::load(&manifest_path)?;
    let tag_hits = scan::scan_repo(&root)?;
    let results = scan::map_requirements(&requirements, &tag_hits);

    match &cli.command {
        Commands::Scan => {
            let total = results.len();
            let covered = results.iter().filter(|r| r.covered).count();
            println!("ASVS coverage: {covered}/{total} requirements covered");
            for r in &results {
                let status = if r.covered { "OK" } else { "MISSING" };
                println!(
                    "  [{status}] {} ({}) - {}",
                    r.requirement.id,
                    r.requirement.level,
                    if r.covered {
                        format!("{} test(s)", r.hits.len())
                    } else {
                        String::from("no tests found")
                    }
                );
            }
        }

        Commands::Report { format, output } => {
            let rendered = match format.as_str() {
                "json" => report::render_json(&results)?,
                "html" => report::render_html(&results),
                "md" | "markdown" => report::render_markdown(&results),
                other => {
                    anyhow::bail!("unknown format '{}'; use html, md, or json", other);
                }
            };
            if let Some(out_path) = output {
                std::fs::write(out_path, &rendered)?;
            } else {
                print!("{rendered}");
            }
        }

        Commands::Check { strict } => {
            let gaps = if *strict {
                scan::gaps(&results)
            } else {
                scan::gaps_by_level(&results, &manifest::Level::L3)
            };

            if gaps.is_empty() {
                println!("All required ASVS requirements are covered.");
            } else {
                eprintln!("ASVS coverage gaps ({}):", gaps.len());
                for g in &gaps {
                    eprintln!(
                        "  MISSING {} ({}) {}",
                        g.requirement.id, g.requirement.level, g.requirement.description
                    );
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
