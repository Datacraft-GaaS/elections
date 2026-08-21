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
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, title) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not Found"),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApiError::InvalidInput(_) => (StatusCode::UNPROCESSABLE_ENTITY, "Invalid Input"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };

        let body = json!({
            "type": "about:blank",
            "title": title,
            "status": status.as_u16(),
            "detail": self.to_string(),
        });

        let mut resp = (status, Json(body)).into_response();
        resp.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            "application/problem+json".parse().unwrap(),
        );
        resp
    }
}
