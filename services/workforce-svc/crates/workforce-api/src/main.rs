use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("workforce_api=info".parse().unwrap()),
        )
        .json()
        .init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8100".to_string());
    info!(http_addr, "workforce-api starting");

    let store = match env::var("DATABASE_URL") {
        Ok(url) => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .connect(&url)
                .await
                .expect("database connect failed");
            // Ensure the transactional-outbox table exists before serving
            // (cross-service events, design §8).
            eel_outbox_rs::Outbox::new(pool.clone())
                .migrate()
                .await
                .expect("outbox migration failed");
            workforce_api::store::Store::Pg(pool)
        }
        Err(_) => {
            info!("DATABASE_URL unset; using in-memory store");
            workforce_api::store::Store::Mem(Default::default())
        }
    };

    let app = workforce_api::router::build_router(store);
    let listener = tokio::net::TcpListener::bind(&http_addr)
        .await
        .expect("bind failed");
    info!(http_addr, "workforce-api HTTP listening");
    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}
