pub mod appeals;
pub mod cases;
pub mod health;
pub mod reports;
pub mod spec;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(cases::router())
        .merge(appeals::router())
        .merge(reports::router())
        .merge(spec::router())
}
