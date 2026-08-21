use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResilienceError {
    #[error("circuit open: dependency {dependency} unavailable")]
    CircuitOpen { dependency: String },
    #[error("bulkhead full: {dependency} at capacity")]
    BulkheadFull { dependency: String },
    #[error("timeout exceeded")]
    Timeout,
    #[error("retries exhausted after {attempts} attempts")]
    RetriesExhausted { attempts: u32 },
    #[error("idempotency conflict: key already used with different body")]
    IdempotencyConflict,
    #[error("service degraded: {mode}")]
    Degraded { mode: String },
}
