use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShedError {
    #[error("service overloaded, retry after {retry_after_secs}s")]
    Overloaded { retry_after_secs: u64 },
    #[error("inner service error")]
    Inner,
}
