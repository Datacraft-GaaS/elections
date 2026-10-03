// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    /// The operation is declared by the contract but has no implementation
    /// behind it. Returned instead of a fabricated success: a caller that gets
    /// 501 knows the answer does not exist, where a caller handed an invented
    /// `match_score` and `outcome` has no way to tell it from a real verdict.
    #[error("{0}")]
    NotImplemented(String),
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
            ApiError::NotImplemented(_) => (StatusCode::NOT_IMPLEMENTED, "Not Implemented"),
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
        | ApiError::Unauthorized(d)
        | ApiError::NotImplemented(d) => d.clone(),
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
        (
            status,
            [(CONTENT_TYPE, "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}
