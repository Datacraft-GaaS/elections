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

//! RFC 9457 `application/problem+json` error surface (ADR-0011).

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

const ERR_BASE: &str = "https://errors.eel.internal";

#[derive(Debug, Serialize)]
struct Problem {
    #[serde(rename = "type")]
    problem_type: String,
    title: String,
    status: u16,
    detail: String,
    instance: String,
}

/// API-level error carrying everything needed to render a Problem document.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    problem_type: &'static str,
    title: &'static str,
    detail: String,
    instance: &'static str,
}

impl ApiError {
    pub fn bad_request(instance: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            problem_type: "common/bad_request",
            title: "Bad Request",
            detail: detail.into(),
            instance,
        }
    }

    pub fn unauthorized(instance: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            problem_type: "common/unauthorized",
            title: "Unauthorized",
            detail: "Bearer token is missing or has expired.".into(),
            instance,
        }
    }

    pub fn not_found(instance: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            problem_type: "common/not_found",
            title: "Not Found",
            detail: "No resource found with the provided identifier.".into(),
            instance,
        }
    }

    pub fn conflict(instance: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            problem_type: "common/conflict",
            title: "Conflict",
            detail: detail.into(),
            instance,
        }
    }

    pub fn already_identified(instance: &'static str, voter_id: &str, session_id: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            problem_type: "bvi/already_identified",
            title: "Conflict",
            detail: format!(
                "Voter {voter_id} has already been identified in session {session_id}."
            ),
            instance,
        }
    }

    pub fn unprocessable(instance: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            problem_type: "bvi/invalid_session_state",
            title: "Unprocessable Entity",
            detail: detail.into(),
            instance,
        }
    }

    pub fn internal(instance: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            problem_type: "common/internal_error",
            title: "Internal Server Error",
            detail: "An unexpected error occurred. Use trace_id to report this incident.".into(),
            instance,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Problem {
            problem_type: format!("{ERR_BASE}/{}", self.problem_type),
            title: self.title.to_owned(),
            status: self.status.as_u16(),
            detail: self.detail,
            instance: self.instance.to_owned(),
        };
        let mut resp = (self.status, Json(body)).into_response();
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/problem+json"),
        );
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn renders_problem_json() {
        let err = ApiError::conflict("/api/v1/bvi/sessions", "active session exists");
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/problem+json"
        );
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["status"], 409);
        assert_eq!(v["title"], "Conflict");
        assert_eq!(v["type"], "https://errors.eel.internal/common/conflict");
        assert_eq!(v["instance"], "/api/v1/bvi/sessions");
    }
}
