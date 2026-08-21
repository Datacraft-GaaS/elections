#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = dereg_appeal::app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
    tracing::info!("dereg-appeal listening on :8083");
    axum::serve(listener, app).await.unwrap();
}
