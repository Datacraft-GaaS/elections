#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/voter_card".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await
        .expect("connect to voter-card database");
    let app = voter_card::app(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8084").await.unwrap();
    tracing::info!("voter-card listening on :8084");
    axum::serve(listener, app).await.unwrap();
}
