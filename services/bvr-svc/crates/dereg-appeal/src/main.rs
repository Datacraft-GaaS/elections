#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/dereg_appeal".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await
        .expect("connect to dereg-appeal database");
    let app = dereg_appeal::app(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
    tracing::info!("dereg-appeal listening on :8083");
    axum::serve(listener, app).await.unwrap();
}
