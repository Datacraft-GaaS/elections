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
    #[error("nonce expired or invalid")]
    NonceExpired,
    #[error("nonce not found")]
    NonceNotFound,
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("certificate chain verification failed: {0}")]
    ChainVerificationFailed(String),
    #[error("report too short: expected {expected} bytes, got {got}")]
    ReportTooShort { expected: usize, got: usize },
    #[error("software attestation rejected in production policy")]
    SoftwareAttestationRejected,
}
