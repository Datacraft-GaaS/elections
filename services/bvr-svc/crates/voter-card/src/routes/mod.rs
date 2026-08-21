pub mod batches;
pub mod blacklist;
pub mod health;
pub mod status;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(status::router())
        .merge(batches::router())
        .merge(blacklist::router())
}
