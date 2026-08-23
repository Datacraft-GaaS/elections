use std::sync::Arc;

use axum::{routing::post, Router};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://eel:eel@localhost/eel_tlog".to_string());
    let api_base = std::env::var("TLOG_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(16)
            .connect(&db_url)
            .await?,
    );

    let state = tlog_witness::handler::WitnessState { pool };
    let app = Router::new()
        .route(
            "/api/v1/transparency/witnesses/cosign",
            post(tlog_witness::handler::post_cosign),
        )
        .with_state(state);

    let iab = Arc::new(tlog_witness::iab_client::IabWitnessClient::from_env(
        api_base,
        "iab-internal",
    )?);
    tokio::spawn(iab.run());

    let addr =
        std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
