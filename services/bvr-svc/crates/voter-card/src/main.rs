#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = voter_card::app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8084").await.unwrap();
    tracing::info!("voter-card listening on :8084");
    axum::serve(listener, app).await.unwrap();
}
