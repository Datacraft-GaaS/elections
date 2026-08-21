use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("rts_publisher=info".parse().unwrap()),
    ).json().init();
    let db_url = env::var("DB_URL").expect("DB_URL must be set");
    let interval_secs: u64 = env::var("PUBLICATION_INTERVAL_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(300);
    info!(interval_secs, "rts-publisher starting");
    let pool = sqlx::PgPool::connect(&db_url).await.expect("failed to connect to DB");
    rts_publisher::scheduler::run_publication_loop(&pool, interval_secs).await;
}
