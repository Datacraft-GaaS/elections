use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tlog_api::{rest, ApiState};
use tlog_sequencer::Sequencer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://eel:eel@localhost/eel_tlog".to_string());

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(64)
            .connect(&db_url)
            .await?,
    );
    let sequencer = Arc::new(Sequencer::new(pool.clone(), 50_000));
    let state = ApiState { pool, sequencer };

    let rest_app = rest::router().with_state(state.clone());

    let addr =
        std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "tlog-api listening");
    axum::serve(listener, rest_app).await?;
    Ok(())
}
