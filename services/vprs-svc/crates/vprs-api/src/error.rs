use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unprocessable entity: {0}")]
    UnprocessableEntity(String),
    #[error("too many requests")]
    TooManyRequests,
    #[error("service unavailable")]
    ServiceUnavailable,
    #[error("gone")]
    Gone,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            ApiError::UnprocessableEntity(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
            ApiError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            ApiError::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ApiError::Gone => (StatusCode::GONE, self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
