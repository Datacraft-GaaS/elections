//! bvi-api binary: REST server for the BVI backend (M06).

use std::env;

use bvi_api::{app, state::AppState, store::PgStore};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .json()
        .init();

    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("database connect failed");

    info!(http_addr, "bvi-api starting");
    let state = AppState::new(PgStore::new(pool));
    let listener = tokio::net::TcpListener::bind(&http_addr)
        .await
        .expect("bind failed");
    info!(http_addr, "bvi-api HTTP listening");
    axum::serve(listener, app(state)).await.expect("server failed");
}
