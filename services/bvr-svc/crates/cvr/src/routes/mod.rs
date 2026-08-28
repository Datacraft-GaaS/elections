pub mod batches;
pub mod events;
pub mod family_notify;
pub mod field_notifications;
pub mod health;
pub mod snapshots;
pub mod spec;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(batches::router())
        .merge(events::router())
        .merge(snapshots::router())
        .merge(family_notify::router())
        .merge(field_notifications::router())
        .merge(spec::router())
}
