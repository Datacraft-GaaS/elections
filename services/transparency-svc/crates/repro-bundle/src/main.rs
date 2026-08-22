//! `eel-repro` CLI: assemble and verify reproducibility bundles (WP-4.6, M14).
//!
//! Subcommands:
//!   - `verify <manifest.json> <dir>`: recompute each component SHA-256 against
//!     files under <dir> and confirm they match the manifest digests.
//!   - `manifest`: print a fresh (unsigned) manifest skeleton for an election.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use repro_bundle::{BundleManifest, ManifestVersion};
use sha2::Digest;
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "eel-repro",
    about = "Reproducibility bundle assembler and verifier"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Verify component digests in a manifest against a directory of files.
    Verify {
        /// Path to the bundle manifest JSON.
        manifest: PathBuf,
        /// Directory holding the referenced component files.
        dir: PathBuf,
    },
    /// Emit a skeleton manifest for an election, then exit.
    Manifest {
        election_id: String,
        /// Optional prior bundle id (correction bundles).
        #[arg(long)]
        prior: Option<Uuid>,
    },
}

fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let data = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let digest = sha2::Sha256::digest(&data);
    Ok(hex::encode(digest))
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Verify { manifest, dir } => {
            let data = std::fs::read(&manifest)
                .with_context(|| format!("read manifest {}", manifest.display()))?;
            let m: BundleManifest =
                serde_json::from_slice(&data).with_context(|| "parse manifest JSON")?;
            let mut files = HashMap::new();
            for entry in &m.components {
                let p = dir.join(&entry.reference);
                let digest = sha256_file(&p)
                    .with_context(|| format!("read component {}", entry.reference))?;
                files.insert(entry.reference.clone(), digest);
            }
            let mut ok = 0usize;
            let mut bad = 0usize;
            for entry in &m.components {
                let actual = files
                    .get(&entry.reference)
                    .ok_or_else(|| anyhow::anyhow!("missing component {}", entry.reference))?;
                let status = if actual == &entry.sha256 {
                    ok += 1;
                    "ok"
                } else {
                    bad += 1;
                    "MISMATCH"
                };
                println!("{status}\t{}\t{}", entry.reference, entry.sha256);
            }
            println!(
                "{} components, {} ok, {} mismatched",
                m.components.len(),
                ok,
                bad
            );
            if bad != 0 {
                anyhow::bail!("bundle verification failed");
            }
        }
        Command::Manifest { election_id, prior } => {
            let m = BundleManifest {
                format_version: ManifestVersion::default(),
                bundle_id: Uuid::new_v4(),
                election_id,
                assembled_at: chrono::Utc::now(),
                merkle_root: String::new(),
                components: Vec::new(),
                build_log_hash: String::new(),
                prior_bundle_id: prior,
                signature: None,
            };
            println!("{}", serde_json::to_string_pretty(&m)?);
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run(cli)
}
