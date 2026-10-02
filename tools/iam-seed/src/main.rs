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

mod keycloak;
mod openbao;
mod spicedb;
mod spire;
mod verify;

pub use iam_seed::compute_diff;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(name = "iam-seed", version, about = "Idempotent IAM seeding CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply Keycloak realm configuration.
    KeycloakApply {
        /// Keycloak realm name.
        #[arg(long)]
        realm: String,
        /// Keycloak base URL (e.g. https://auth.eel.internal).
        #[arg(long)]
        url: String,
        /// Path to realm JSON file. Defaults to platform/keycloak/<realm>.json.
        #[arg(long)]
        realm_file: Option<String>,
        /// Apply changes without prompting.
        #[arg(long)]
        yes: bool,
    },
    /// Apply SpiceDB schema.
    SpicedbApply {
        /// SpiceDB gRPC endpoint (e.g. spicedb.eel.internal:50051).
        #[arg(long)]
        endpoint: String,
        /// SpiceDB API token.
        #[arg(long, env = "SPICEDB_TOKEN")]
        token: String,
        /// Directory containing schema .zed files. Defaults to platform/spicedb/schema/.
        #[arg(long)]
        schema_dir: Option<String>,
        /// Apply changes without prompting.
        #[arg(long)]
        yes: bool,
    },
    /// Apply SPIRE workload entries.
    SpireApply {
        /// SPIRE server address (e.g. spire-server.eel.internal:8081).
        #[arg(long)]
        server_url: String,
        /// Directory containing entry YAML files. Defaults to platform/spire/entries/.
        #[arg(long)]
        entries_dir: Option<String>,
        /// Apply changes without prompting.
        #[arg(long)]
        yes: bool,
    },
    /// Apply OpenBao policies and mounts.
    OpenbaoApply {
        /// OpenBao address (e.g. https://bao.eel.internal).
        #[arg(long, env = "VAULT_ADDR")]
        url: String,
        /// OpenBao root or admin token.
        #[arg(long, env = "VAULT_TOKEN")]
        token: String,
        /// Directory containing .hcl policy files. Defaults to platform/openbao/policies/.
        #[arg(long)]
        policies_dir: Option<String>,
        /// Directory containing mount YAML files. Defaults to platform/openbao/mounts/.
        #[arg(long)]
        mounts_dir: Option<String>,
        /// Apply changes without prompting.
        #[arg(long)]
        yes: bool,
    },
    /// Run all apply subcommands in sequence.
    All {
        /// Target environment.
        #[arg(long, value_parser = ["dev", "staging", "pre-prod", "prod"])]
        env: String,
        /// Apply changes without prompting.
        #[arg(long)]
        yes: bool,
    },
    /// Verify IAM state against golden fixtures.
    Verify {
        /// Target environment to verify.
        #[arg(long, value_parser = ["dev", "staging", "pre-prod", "prod"])]
        env: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("iam_seed=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::KeycloakApply { realm, url, realm_file, yes } => {
            keycloak::apply(&realm, &url, realm_file.as_deref(), yes).await?;
        }
        Commands::SpicedbApply { endpoint, token, schema_dir, yes } => {
            spicedb::apply(&endpoint, &token, schema_dir.as_deref(), yes).await?;
        }
        Commands::SpireApply { server_url, entries_dir, yes } => {
            spire::apply(&server_url, entries_dir.as_deref(), yes).await?;
        }
        Commands::OpenbaoApply { url, token, policies_dir, mounts_dir, yes } => {
            openbao::apply(&url, &token, policies_dir.as_deref(), mounts_dir.as_deref(), yes)
                .await?;
        }
        Commands::All { env, yes } => {
            info!(env = %env, "running all apply subcommands");
            openbao::apply(
                &format!("https://bao.{}.eel.internal", env),
                &std::env::var("VAULT_TOKEN").unwrap_or_default(),
                Some("platform/openbao/policies"),
                Some("platform/openbao/mounts"),
                yes,
            )
            .await?;
            keycloak::apply(
                "eel",
                &format!("https://auth.{}.eel.internal", env),
                None,
                yes,
            )
            .await?;
            spicedb::apply(
                &format!("spicedb.{}.eel.internal:50051", env),
                &std::env::var("SPICEDB_TOKEN").unwrap_or_default(),
                Some("platform/spicedb/schema"),
                yes,
            )
            .await?;
            spire::apply(
                &format!("spire-server.{}.eel.internal:8081", env),
                Some("platform/spire/entries"),
                yes,
            )
            .await?;
        }
        Commands::Verify { env } => {
            let exit_code = verify::run(env.as_deref()).await?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
    }

    Ok(())
}
