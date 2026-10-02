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
