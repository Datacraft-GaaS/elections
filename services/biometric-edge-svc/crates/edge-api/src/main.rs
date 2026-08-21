#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = edge_api::app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8090").await.unwrap();
    tracing::info!("edge-api listening on :8090");
    axum::serve(listener, app).await.unwrap();
}
