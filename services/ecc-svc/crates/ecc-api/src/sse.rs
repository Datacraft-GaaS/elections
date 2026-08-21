use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};

fn sse_stub_response(portal: &str) -> Response {
    let body = format!("data: {{\"portal\":\"{portal}\",\"status\":\"connected\"}}\n\n");
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header("X-Accel-Buffering", "no")
        .body(axum::body::Body::from(body))
        .unwrap()
}

pub async fn party_stream() -> impl IntoResponse {
    sse_stub_response("party")
}

pub async fn observer_stream() -> impl IntoResponse {
    sse_stub_response("observer")
}

pub async fn journalist_stream() -> impl IntoResponse {
    sse_stub_response("journalist")
}
