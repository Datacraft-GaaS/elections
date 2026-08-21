use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretsError {
    #[error("vault client error: {0}")]
    VaultClient(String),
    #[error("auth failed: {0}")]
    Auth(String),
    #[error("lease expired")]
    LeaseExpired,
    #[error("transit error: {0}")]
    Transit(String),
    #[error("pki error: {0}")]
    Pki(String),
    #[error("admin cred cap exceeded: max 4h")]
    AdminCapExceeded,
    #[error("database error: {0}")]
    Database(String),
}
