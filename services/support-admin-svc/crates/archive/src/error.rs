use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Domain errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid state transition: {detail}")]
    InvalidTransition { detail: String },

    #[error("integrity violation: {detail}")]
    IntegrityViolation { detail: String },
}

// ---------------------------------------------------------------------------
// HTTP API errors (RFC 7807 problem detail)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub title: &'static str,
    pub detail: String,
}

impl ApiError {
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            title: "Not Found",
            detail: detail.into(),
        }
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            title: "Bad Request",
            detail: detail.into(),
        }
    }

    pub fn unprocessable(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            title: "Unprocessable Entity",
            detail: detail.into(),
        }
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            title: "Internal Server Error",
            detail: detail.into(),
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::InvalidTransition { .. } => Self::unprocessable(e.to_string()),
            DomainError::IntegrityViolation { .. } => Self::unprocessable(e.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({
            "type": "about:blank",
            "title": self.title,
            "status": self.status.as_u16(),
            "detail": self.detail,
        });
        (self.status, Json(body)).into_response()
    }
}
