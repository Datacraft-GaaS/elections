pub mod batches;
pub mod blacklist;
pub mod spec;
pub mod health;
pub mod status;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(status::router())
        .merge(batches::router())
        .merge(blacklist::router())
        .merge(spec::router())
}
