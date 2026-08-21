pub mod appeals;
pub mod cases;
pub mod health;
pub mod reports;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(cases::router())
        .merge(appeals::router())
        .merge(reports::router())
}
