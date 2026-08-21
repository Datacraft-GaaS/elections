pub mod adjudication;
pub mod enrolments;
pub mod handoff;
pub mod health;
pub mod merkle;
pub mod registers;
pub mod sar;
pub mod transfers;
pub mod voters;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(enrolments::router())
        .merge(voters::router())
        .merge(registers::router())
        .merge(transfers::router())
        .merge(sar::router())
        .merge(merkle::router())
        .merge(adjudication::router())
        .merge(handoff::router())
}
