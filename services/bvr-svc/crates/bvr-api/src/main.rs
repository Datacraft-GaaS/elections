#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = bvr_api::app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("bvr-api listening on :8080");
    axum::serve(listener, app).await.unwrap();
}
