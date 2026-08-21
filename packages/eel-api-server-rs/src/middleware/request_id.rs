use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn inject_request_id(mut req: Request<Body>, next: Next) -> Response {
	let id = req
		.headers()
		.get(REQUEST_ID_HEADER)
		.and_then(|v| v.to_str().ok())
		.map(str::to_owned)
		.unwrap_or_else(|| Uuid::new_v4().to_string());

	req.extensions_mut().insert(RequestId(id.clone()));

	let mut resp = next.run(req).await;
	resp.headers_mut()
		.insert(REQUEST_ID_HEADER, id.parse().unwrap());
	resp
}

#[derive(Clone, Debug)]
pub struct RequestId(pub String);
