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

mod api_client;
mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "eel-report",
    about = "Operator CLI for ABIS report management",
    version
)]
struct Cli {
    /// Reports API base URL
    #[arg(long, env = "EEL_REPORTS_URL", default_value = "http://localhost:8080")]
    api_url: String,

    /// Bearer token for auth (fallback when SPIRE socket unavailable)
    #[arg(long, env = "EEL_REPORTS_TOKEN")]
    token: Option<String>,

    /// SPIRE agent socket path
    #[arg(long, env = "SPIRE_AGENT_SOCKET", default_value = "/run/spire/sockets/agent.sock")]
    spire_socket: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List report definitions
    List {
        /// Filter by report class (operational, statistical, regulatory, legal, public, financial, executive, ad_hoc)
        #[arg(long)]
        class: Option<String>,
        /// Show only active (non-retired) definitions
        #[arg(long, default_value_t = false)]
        active_only: bool,
    },
    /// Create a new report definition from a YAML file
    Define {
        /// Path to definition YAML file
        #[arg(long)]
        file: String,
    },
    /// Trigger a report run
    Run {
        /// Report slug
        #[arg(long)]
        slug: String,
        /// Window start (ISO 8601)
        #[arg(long, value_name = "ISO")]
        from: String,
        /// Window end (ISO 8601)
        #[arg(long, value_name = "ISO")]
        to: String,
        /// Output formats (comma-separated: pdf,csv,json,html,xlsx,png)
        #[arg(long, value_delimiter = ',', default_values = ["pdf", "csv", "json"])]
        format: Vec<String>,
        /// Wait for the run to complete
        #[arg(long, default_value_t = false)]
        wait: bool,
    },
    /// Get the status of a report run
    Status {
        /// Run UUID
        #[arg(long)]
        run_id: String,
    },
    /// Download a report artifact
    Download {
        /// Artifact UUID
        #[arg(long)]
        artifact_id: String,
        /// Output file path
        #[arg(long, value_name = "PATH")]
        out: String,
        /// Verify SHA-256 after download
        #[arg(long, default_value_t = false)]
        verify: bool,
    },
    /// Verify a downloaded artifact against the catalog
    Verify {
        /// Local file to verify
        #[arg(long)]
        file: String,
        /// Artifact UUID to compare against
        #[arg(long)]
        artifact_id: String,
    },
    /// Create a subscription
    Subscribe {
        /// Subscriber name
        #[arg(long)]
        subscriber: String,
        /// Report slug
        #[arg(long)]
        slug: String,
        /// Preferred format
        #[arg(long, default_value = "pdf")]
        format: String,
        /// Delivery channel (email, webhook, kafka, sftp, gov_printer, kbc_sftp, api_pull)
        #[arg(long, default_value = "email")]
        channel: String,
        /// Channel endpoint (email address, webhook URL, topic, etc.)
        #[arg(long)]
        endpoint: String,
    },
    /// List deliveries for a run
    Deliveries {
        /// Run UUID
        #[arg(long, value_name = "UUID")]
        run: String,
        /// Retry all failed deliveries
        #[arg(long, default_value_t = false)]
        retry_failed: bool,
    },
    /// Show the next N scheduled runs for a slug
    Schedule {
        /// Report slug
        #[arg(long)]
        slug: String,
        /// Number of upcoming runs to show
        #[arg(long, default_value_t = 10)]
        next: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = api_client::ReportsClient::new(&cli.api_url, cli.token.as_deref(), &cli.spire_socket)?;

    match cli.command {
        Commands::List { class, active_only } => {
            commands::list::run(&client, class.as_deref(), active_only).await
        }
        Commands::Define { file } => {
            commands::define::run(&client, &file).await
        }
        Commands::Run { slug, from, to, format, wait } => {
            commands::run::run(&client, &slug, &from, &to, &format, wait).await
        }
        Commands::Status { run_id } => {
            commands::status::run(&client, &run_id).await
        }
        Commands::Download { artifact_id, out, verify } => {
            commands::download::run(&client, &artifact_id, &out, verify).await
        }
        Commands::Verify { file, artifact_id } => {
            commands::verify::run(&client, &file, &artifact_id).await
        }
        Commands::Subscribe { subscriber, slug, format, channel, endpoint } => {
            commands::subscribe::run(&client, &subscriber, &slug, &format, &channel, &endpoint).await
        }
        Commands::Deliveries { run, retry_failed } => {
            commands::deliveries::run(&client, &run, retry_failed).await
        }
        Commands::Schedule { slug, next } => {
            commands::schedule::run(&client, &slug, next).await
        }
    }
}
