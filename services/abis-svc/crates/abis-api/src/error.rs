//! RFC 9457 problem+json error mapping for API handlers.

use axum::{
	http::{header::CONTENT_TYPE, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("{0}")]
	Validation(String),
	#[error("{0}")]
	NotFound(String),
	#[error("{0}")]
	Conflict(String),
	#[error("{0}")]
	Unauthorized(String),
	#[error("abis store unavailable")]
	Store(#[source] sqlx::Error),
}

impl ApiError {
	fn status_title(&self) -> (StatusCode, &'static str) {
		match self {
			ApiError::Validation(_) => (StatusCode::BAD_REQUEST, "Bad Request"),
			ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "Not Found"),
			ApiError::Conflict(_) => (StatusCode::CONFLICT, "Conflict"),
			ApiError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
			// Store errors are infrastructure failures; never leak internals.
			ApiError::Store(_) => (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable"),
		}
	}
}

impl From<sqlx::Error> for ApiError {
	fn from(e: sqlx::Error) -> Self {
		let msg = e.to_string();
		if msg.contains("unique") || msg.contains("duplicate key") {
			return ApiError::Conflict("resource already exists".to_string());
		}
		ApiError::Store(e)
	}
}

fn detail(err: &ApiError) -> String {
	match err {
		ApiError::Validation(d)
		| ApiError::NotFound(d)
		| ApiError::Conflict(d)
		| ApiError::Unauthorized(d) => d.clone(),
		ApiError::Store(_) => "ABIS store temporarily unavailable".to_string(),
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		let (status, title) = self.status_title();
		let body = json!({
			"type": "about:blank",
			"title": title,
			"status": status.as_u16(),
			"detail": detail(&self),
		});
		tracing::warn!(status = status.as_u16(), detail = %body["detail"], "request rejected");
		(status, [(CONTENT_TYPE, "application/problem+json")], Json(body)).into_response()
	}
}
