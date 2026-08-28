pub mod accreditation;
pub mod cms;
pub mod exam;
pub mod lms;
pub mod medical;
pub mod spec;

use axum::{http::StatusCode, response::IntoResponse};

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}
