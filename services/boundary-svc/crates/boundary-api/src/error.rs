use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

/// RFC 7807 problem detail returned on all 4xx/5xx responses.
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

    pub fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            title: "Internal Server Error",
            detail: "an unexpected error occurred".to_owned(),
        }
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            title: "Forbidden",
            detail: detail.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let trace_id = Uuid::now_v7();
        let body = json!({
            "type": format!("https://eel.local/problems/{}", self.status.as_u16()),
            "title": self.title,
            "status": self.status.as_u16(),
            "detail": self.detail,
            "instance": format!("/api/v1/boundary/errors/{trace_id}"),
            "eel_trace_id": trace_id.to_string(),
        });
        (self.status, Json(body)).into_response()
    }
}

impl From<boundary_domain::types::DomainError> for ApiError {
    fn from(e: boundary_domain::types::DomainError) -> Self {
        match e {
            boundary_domain::types::DomainError::NotFound { code } => {
                Self::not_found(format!("boundary {code} not found"))
            }
            boundary_domain::types::DomainError::UnsignedPolygon { station_code } => {
                Self::unprocessable(format!("polygon for {station_code} is unsigned"))
            }
            other => Self::bad_request(other.to_string()),
        }
    }
}
