use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("OTel init failed: {0}")]
    Init(String),
    #[error("SQL migration failed: {0}")]
    SqlMigration(String),
    #[error("PII serialization blocked")]
    PiiBlocked,
}
