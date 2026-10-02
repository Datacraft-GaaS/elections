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

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tlog-admin", about = "eel transparency log admin CLI (WP-2.1)")]
struct Cli {
    #[arg(long, default_value = "http://localhost:8080")]
    api_url: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print latest STH for a domain.
    Checkpoint {
        #[arg(long)]
        domain: String,
    },
    /// Export tree snapshot to MinIO/S3 or a local path.
    Export {
        #[arg(long)]
        to: String,
        /// Database URL; falls back to $DATABASE_URL.
        #[arg(long)]
        db_url: Option<String>,
    },
    /// Verify consistency between two STHs.
    VerifyChain {
        #[arg(long)]
        sth_a: u64,
        #[arg(long)]
        sth_b: u64,
    },
    /// Check all witnesses for split views.
    WitnessCrossCheck,
    /// List parked ingest failures (non-zero exit count = open failures).
    IngestFailures {
        /// Database URL; falls back to $DATABASE_URL.
        #[arg(long)]
        db_url: Option<String>,
    },
    /// Acknowledge a failure as replayed.
    IngestFailureAck {
        #[arg(long)]
        id: uuid::Uuid,
        /// Database URL; falls back to $DATABASE_URL.
        #[arg(long)]
        db_url: Option<String>,
    },
    /// Replay a Kafka topic into the log.
    BulkIngest {
        #[arg(long)]
        topic: String,
        #[arg(long, default_value = "0")]
        from_offset: i64,
    },
    /// Sign and print an integrity statement over the current log state.
    IntegrityStatement {
        /// Database URL; falls back to $DATABASE_URL.
        #[arg(long)]
        db_url: Option<String>,
        /// 32-byte Ed25519 seed, hex.
        #[arg(long)]
        signing_key_hex: String,
    },
    /// Sign a coverage certificate diffing expected vs present station ids.
    Coverage {
        /// Line-delimited expected station/form ids.
        #[arg(long)]
        expected_file: String,
        /// Line-delimited ids actually present in the log.
        #[arg(long)]
        present_file: String,
        /// 32-byte Ed25519 seed, hex.
        #[arg(long)]
        signing_key_hex: String,
    },
    /// Sign an artifact attestation (sha256 + Rekor UUID).
    AttestArtifact {
        #[arg(long)]
        artifact_sha256: String,
        #[arg(long)]
        rekor_uuid: String,
        #[arg(long)]
        signer_name: String,
        /// 32-byte Ed25519 seed, hex.
        #[arg(long)]
        signing_key_hex: String,
    },
    /// Verify a signed attestation file against a public key.
    AttestVerify {
        #[arg(long)]
        attestation_file: String,
        /// Hex verifying key; defaults to the embedded signer_pubkey.
        #[arg(long)]
        pubkey_hex: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Checkpoint { domain } => {
            tlog_admin::checkpoint::run(&domain, &cli.api_url).await?
        }
        Commands::Export { to, db_url } => {
            let db = db_url
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .expect("export needs --db-url or $DATABASE_URL");
            tlog_admin::export::run(&db, &to).await?
        }
        Commands::VerifyChain { sth_a, sth_b } => {
            tlog_admin::verify::run(sth_a, sth_b, &cli.api_url).await?
        }
        Commands::WitnessCrossCheck => tlog_admin::witness::run(&cli.api_url).await?,
        Commands::IngestFailures { db_url } => {
            let db = db_url
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .expect("ingest-failures needs --db-url or $DATABASE_URL");
            let n = tlog_admin::failures::list(&db).await?;
            if n > 0 {
                anyhow::bail!("public log is missing {n} event(s); the log must not be trusted as complete");
            }
        }
        Commands::IngestFailureAck { id, db_url } => {
            let db = db_url
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .expect("ingest-failure-ack needs --db-url or $DATABASE_URL");
            tlog_admin::failures::ack(&db, id).await?
        }
        Commands::BulkIngest { topic, from_offset } => {
            tlog_admin::bulk_ingest::run(&topic, from_offset, &cli.api_url).await?
        }
        Commands::IntegrityStatement { db_url, signing_key_hex } => {
            let db = db_url
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .expect("integrity-statement needs --db-url or $DATABASE_URL");
            let open = tlog_admin::statement::run(&db, &signing_key_hex).await?;
            if open > 0 {
                anyhow::bail!("integrity statement records {open} open ingest failure(s); the log must not be trusted as complete");
            }
        }
        Commands::Coverage { expected_file, present_file, signing_key_hex } => {
            let missing = tlog_admin::coverage::run(&expected_file, &present_file, &signing_key_hex).await?;
            if missing > 0 {
                anyhow::bail!("coverage certificate shows {missing} missing station/form id(s); the log must not be trusted as complete");
            }
        }
        Commands::AttestArtifact { artifact_sha256, rekor_uuid, signer_name, signing_key_hex } => {
            tlog_admin::attestation::run(&artifact_sha256, &rekor_uuid, &signer_name, &signing_key_hex).await?
        }
        Commands::AttestVerify { attestation_file, pubkey_hex } => {
            let ok = tlog_admin::attestation::run_verify(&attestation_file, pubkey_hex.as_deref().unwrap_or("")).await?;
            if !ok {
                anyhow::bail!("attestation verification FAILED");
            }
            println!("attestation verified OK");
        }
    }
    Ok(())
}
