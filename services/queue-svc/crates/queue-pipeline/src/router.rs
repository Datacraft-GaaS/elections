use axum::{routing::get, routing::post, Router};

use crate::handlers::{health, post_snapshot};

pub fn build_router() -> Router {
    Router::new().route("/v1/queue/health", get(health)).route(
        "/v1/queue/stations/{station_code}/snapshot",
        post(post_snapshot),
    )
}
