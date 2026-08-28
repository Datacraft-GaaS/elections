//! RFC 9457 `application/problem+json` error responses (ADR-0011).

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use uuid::Uuid;

pub const PROBLEM_BASE: &str = "https://eel.local/problems/ecc-api";

/// RFC 9457 problem document.
#[derive(Debug, Serialize)]
pub struct Problem {
    #[serde(rename = "type")]
    pub type_uri: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: String,
    pub eel_trace_id: String,
}

/// Build a problem document for `status` with a problem-type slug.
pub fn problem(status: StatusCode, type_slug: &str, title: &str, detail: impl Into<String>, instance: &str) -> Problem {
    Problem {
        type_uri: format!("{PROBLEM_BASE}/{type_slug}"),
        title: title.to_string(),
        status: status.as_u16(),
        detail: detail.into(),
        instance: instance.to_string(),
        eel_trace_id: Uuid::now_v7().to_string(),
    }
}

impl Problem {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let mut response = (status, axum::Json(self)).into_response();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, "application/problem+json".parse().expect("static content type"));
        response
    }
}

pub fn bad_request(detail: impl Into<String>, instance: &str) -> Problem {
    problem(StatusCode::BAD_REQUEST, "bad-request", "Malformed request", detail, instance)
}

pub fn not_found(detail: impl Into<String>, instance: &str) -> Problem {
    problem(StatusCode::NOT_FOUND, "not-found", "Resource not found", detail, instance)
}

pub fn conflict(detail: impl Into<String>, instance: &str) -> Problem {
    problem(StatusCode::CONFLICT, "conflict", "Conflict", detail, instance)
}

pub fn unprocessable(detail: impl Into<String>, instance: &str) -> Problem {
    problem(
        StatusCode::UNPROCESSABLE_ENTITY,
        "unprocessable-entity",
        "Semantically invalid request body",
        detail,
        instance,
    )
}

pub fn internal(detail: impl Into<String>, instance: &str) -> Problem {
    problem(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal-server-error",
        "Unexpected server error",
        detail,
        instance,
    )
}
