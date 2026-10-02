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

use serde::{Deserialize, Serialize};

/// RFC 9457 problem type URI prefix. Real services append their slug.
pub const PROBLEM_TYPE_BASE: &str = "https://eel.internal/problems/";

/// A well-known problem type URI from openapi-contracts/_shared/problem-types.yaml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProblemType(pub String);

impl ProblemType {
    pub fn new(slug: &str) -> Self {
        Self(format!("{PROBLEM_TYPE_BASE}{slug}"))
    }

    pub fn unauthorized() -> Self {
        Self::new("unauthorized")
    }
    pub fn forbidden() -> Self {
        Self::new("forbidden")
    }
    pub fn not_found() -> Self {
        Self::new("not-found")
    }
    pub fn unprocessable() -> Self {
        Self::new("unprocessable")
    }
    pub fn conflict() -> Self {
        Self::new("conflict")
    }
    pub fn write_locked() -> Self {
        Self::new("write-locked")
    }
    pub fn idempotency_mismatch() -> Self {
        Self::new("idempotency-mismatch")
    }
    pub fn internal() -> Self {
        Self::new("internal")
    }
}

/// RFC 9457 problem detail object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    #[serde(rename = "type")]
    pub problem_type: ProblemType,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl Problem {
    pub fn new(problem_type: ProblemType, title: impl Into<String>, status: u16) -> Self {
        Self {
            problem_type,
            title: title.into(),
            status,
            detail: None,
            instance: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    pub fn unauthorized(detail: impl Into<String>) -> Self {
        Self::new(ProblemType::unauthorized(), "Unauthorized", 401).with_detail(detail)
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self::new(ProblemType::forbidden(), "Forbidden", 403).with_detail(detail)
    }

    pub fn write_locked() -> Self {
        Self::new(
            ProblemType::write_locked(),
            "Write endpoints locked (election-day mode)",
            503,
        )
    }

    pub fn idempotency_mismatch() -> Self {
        Self::new(
            ProblemType::idempotency_mismatch(),
            "Idempotency-Key reused with different body",
            422,
        )
    }

    pub fn to_axum_response(&self) -> axum::response::Response {
        use axum::response::IntoResponse;
        use http::StatusCode;
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_string(self).unwrap_or_default();
        (
            status,
            [(http::header::CONTENT_TYPE, "application/problem+json")],
            body,
        )
            .into_response()
    }
}
