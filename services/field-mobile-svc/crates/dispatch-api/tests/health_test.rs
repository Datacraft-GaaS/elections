use axum::body::Body;
use axum::http::{Request, StatusCode};
use dispatch_api::{app, state::AppState};
use tower::ServiceExt;

#[tokio::test]
async fn health_check_returns_200() {
	let app = app(AppState::default());
	let response = app
		.oneshot(
			Request::builder()
				.uri("/healthz")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
}
