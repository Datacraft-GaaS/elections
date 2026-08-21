use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttestError {
    #[error("attestation quote invalid: {0}")]
    InvalidQuote(String),
    #[error("attestation replay detected")]
    Replay,
    #[error("attestation profile not supported: {0}")]
    UnsupportedProfile(String),
    #[error("enclave quarantined")]
    Quarantined,
}
