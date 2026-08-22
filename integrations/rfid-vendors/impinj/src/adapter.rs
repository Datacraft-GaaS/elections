//! Impinj adapter module: re-exports the handler and domain types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("circuit breaker open")]
    CircuitOpen,
    #[error("upstream HTTP error: status={status}")]
    Upstream { status: u16 },
    #[error("network error: {0}")]
    Network(String),
}

#[derive(Debug, Clone)]
pub struct RegistrationResult {
    pub reader_id: String,
}

#[derive(Debug, Clone)]
pub struct ReadResult {
    pub tag_epc: String,
    pub accepted: bool,
    pub tamper_detected: bool,
}

#[derive(Debug, Clone)]
pub struct ChallengeResponse {
    pub nonce_hex: String,
    pub challenge_hex: String,
}

#[path = "../adapter/handler.rs"]
mod handler;

pub use handler::Handler as ImpinjHandler;
