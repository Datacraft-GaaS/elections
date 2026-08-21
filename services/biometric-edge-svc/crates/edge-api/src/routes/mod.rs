pub mod attestation;
pub mod config;
pub mod distress;
pub mod enrolment;
pub mod health;
pub mod identification;
pub mod mrz;
pub mod ota;
pub mod sync;
pub mod telemetry;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(enrolment::router())
        .merge(identification::router())
        .merge(mrz::router())
        .merge(attestation::router())
        .merge(config::router())
        .merge(sync::router())
        .merge(ota::router())
        .merge(telemetry::router())
        .merge(distress::router())
}
