use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8082));
    tracing::info!("ai-gateway listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, ai_gateway::app()).await.unwrap();
}
