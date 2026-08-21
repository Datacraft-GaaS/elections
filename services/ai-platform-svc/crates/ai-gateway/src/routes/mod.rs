pub mod anomaly;
pub mod decision_log;
pub mod registry;

use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/registry", registry::router())
        .nest("/anomaly", anomaly::router())
        .nest("/decision-log", decision_log::router())
}
