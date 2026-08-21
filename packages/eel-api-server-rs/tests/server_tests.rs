use axum::Router;
use eel_api_server_rs::ApiServer;

#[tokio::test]
async fn healthz_returns_ok() {
	let server = ApiServer::builder()
		.jwks_url("http://localhost/jwks")
		.otel_service_name("test-svc")
		.bind_addr("127.0.0.1:0")
		.build()
		.expect("builder should succeed");

	let router = server.into_router(Router::new());
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let addr = listener.local_addr().unwrap();

	tokio::spawn(async move {
		axum::serve(listener, router).await.unwrap();
	});

	let resp = reqwest::get(format!("http://{addr}/healthz"))
		.await
		.expect("request should succeed");
	assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn readyz_returns_ok() {
	let server = ApiServer::builder()
		.jwks_url("http://localhost/jwks")
		.otel_service_name("test-svc")
		.build()
		.expect("builder should succeed");

	let router = server.into_router(Router::new());
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let addr = listener.local_addr().unwrap();

	tokio::spawn(async move {
		axum::serve(listener, router).await.unwrap();
	});

	let resp = reqwest::get(format!("http://{addr}/readyz"))
		.await
		.expect("request should succeed");
	assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn signed_json_response_has_signature_header() {
	use eel_api_server_rs::response::SignedJson;
	use axum::response::IntoResponse;
	use serde_json::json;

	let resp = SignedJson::ok(json!({"hello": "world"})).into_response();
	assert!(resp.headers().contains_key("x-eel-signature"));
}
