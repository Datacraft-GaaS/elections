#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = cvr::app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
    tracing::info!("cvr listening on :8082");
    axum::serve(listener, app).await.unwrap();
}
