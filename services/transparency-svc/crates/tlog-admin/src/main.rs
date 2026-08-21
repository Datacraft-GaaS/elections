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
    /// Export tree snapshot to MinIO/S3.
    Export {
        #[arg(long)]
        to: String,
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
    /// Replay a Kafka topic into the log.
    BulkIngest {
        #[arg(long)]
        topic: String,
        #[arg(long, default_value = "0")]
        from_offset: i64,
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
        Commands::Export { to } => tlog_admin::export::run(&to).await?,
        Commands::VerifyChain { sth_a, sth_b } => {
            tlog_admin::verify::run(sth_a, sth_b, &cli.api_url).await?
        }
        Commands::WitnessCrossCheck => tlog_admin::witness::run(&cli.api_url).await?,
        Commands::BulkIngest { topic, from_offset } => {
            tlog_admin::bulk_ingest::run(&topic, from_offset, &cli.api_url).await?
        }
    }
    Ok(())
}
