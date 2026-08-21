use archive::routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "archive=info".into()),
        )
        .json()
        .init();

    let app = routes::router();

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("archive service listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
