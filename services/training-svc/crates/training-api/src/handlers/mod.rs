pub mod accreditation;
pub mod cms;
pub mod lms;
pub mod medical;

use axum::{http::StatusCode, response::IntoResponse};

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}
