#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/cvr".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await
        .expect("connect to cvr database");
    let app = cvr::app(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
    tracing::info!("cvr listening on :8082");
    axum::serve(listener, app).await.unwrap();
}
