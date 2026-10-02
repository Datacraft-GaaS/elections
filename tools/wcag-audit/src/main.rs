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

use eel_wcag_audit_lib::{audit, manifest, report};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "eel-wcag-audit",
    about = "WCAG 2.2 AA/AAA compliance auditor for eel UI surfaces",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run axe-core against every URL in the manifest
    Audit {
        /// Path to manifest YAML (default: tools/wcag-audit/manifest.yaml)
        #[arg(long, default_value = "tools/wcag-audit/manifest.yaml")]
        manifest: PathBuf,

        /// Directory to write reports into
        #[arg(long, default_value = "wcag-report")]
        output_dir: PathBuf,

        /// Output format: json | html | both
        #[arg(long, default_value = "both")]
        format: String,

        /// Directory containing pre-recorded axe JSON results (one file per URL slug)
        #[arg(long, env = "EEL_WCAG_STUB_DIR")]
        stub: Option<PathBuf>,
    },

    /// Render a report from a previously generated JSON result file
    Report {
        /// Path to JSON result file produced by `audit`
        #[arg(long)]
        input: PathBuf,

        /// Output format: json | html
        #[arg(long, default_value = "html")]
        format: String,
    },

    /// Verify a JSON result file; exit non-zero if any violations are found
    Check {
        /// Path to JSON result file produced by `audit`
        #[arg(long)]
        input: PathBuf,

        /// Fail on errors (unreachable URLs) as well as violations
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit {
            manifest: manifest_path,
            output_dir,
            format,
            stub,
        } => {
            let mf = manifest::load(&manifest_path)?;
            let stub_dir = stub.as_deref();
            let results = audit::audit_all(&mf.urls, stub_dir);
            let rep = report::Report::from_results(results);

            std::fs::create_dir_all(&output_dir)?;

            let has_violations = rep.has_violations();

            match format.as_str() {
                "json" => {
                    report::write_json(&rep, &output_dir.join("wcag-report.json"))?;
                    eprintln!("wrote {}/wcag-report.json", output_dir.display());
                }
                "html" => {
                    report::write_html(&rep, &output_dir.join("wcag-report.html"))?;
                    eprintln!("wrote {}/wcag-report.html", output_dir.display());
                }
                "both" | _ => {
                    report::write_json(&rep, &output_dir.join("wcag-report.json"))?;
                    report::write_html(&rep, &output_dir.join("wcag-report.html"))?;
                    eprintln!(
                        "wrote {}/wcag-report.json and wcag-report.html",
                        output_dir.display()
                    );
                }
            }

            let s = &rep.summary;
            eprintln!(
                "summary: {} total, {} passing, {} failing, {} errors, {} violations",
                s.total_urls, s.passing, s.failing, s.errors, s.violations_total
            );

            if has_violations {
                std::process::exit(1);
            }
        }

        Commands::Report { input, format } => {
            let json = std::fs::read_to_string(&input)?;
            let rep: report::Report = serde_json::from_str(&json)?;
            match format.as_str() {
                "json" => println!("{}", report::render_json(&rep)?),
                "html" | _ => println!("{}", report::render_html(&rep)),
            }
        }

        Commands::Check { input, strict } => {
            let json = std::fs::read_to_string(&input)?;
            let rep: report::Report = serde_json::from_str(&json)?;
            let s = &rep.summary;
            let fail = s.violations_total > 0 || (strict && s.errors > 0);
            eprintln!(
                "check: {} violations, {} errors (strict={})",
                s.violations_total, s.errors, strict
            );
            if fail {
                bail!(
                    "{} violation(s) found across {} URL(s)",
                    s.violations_total,
                    s.failing
                );
            }
            eprintln!("all URLs pass WCAG checks");
        }
    }

    Ok(())
}
