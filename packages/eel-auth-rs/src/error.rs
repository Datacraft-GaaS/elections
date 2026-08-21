use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("JWT verification failed: {0}")]
    JwtVerification(String),
    #[error("JWKS fetch failed: {0}")]
    JwksFetch(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Unknown kid: {kid}")]
    UnknownKid { kid: String },
    #[error("SpiceDB error: {0}")]
    SpiceDb(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Audit sink error: {0}")]
    AuditSink(String),
    #[error("Transport error: {0}")]
    Transport(String),
}
