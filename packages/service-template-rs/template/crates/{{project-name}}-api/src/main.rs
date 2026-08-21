use eel_api_server_rs::ApiServer;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	fmt().with_env_filter(EnvFilter::from_default_env()).init();

	let server = ApiServer::builder()
		.otel_service_name("{{project-name}}")
		.build()?;

	server
		.serve({{project_name}}_api::routes::router())
		.await?;

	Ok(())
}
