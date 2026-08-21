use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("rts_tally=info".parse().unwrap()),
    ).json().init();
    let _kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let _tlog_endpoint = env::var("TLOG_ENDPOINT").unwrap_or_else(|_| "http://localhost:50052".to_string());
    let _db_url = env::var("DB_URL").expect("DB_URL must be set");
    info!("rts-tally starting (stub mode)");
    tokio::signal::ctrl_c().await.ok();
    info!("rts-tally shutting down");
}
