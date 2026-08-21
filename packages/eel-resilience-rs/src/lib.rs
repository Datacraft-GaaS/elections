//! Resilience primitives for the eel platform (ADR-0009).
//!
//! - Circuit breaker: Closed/Open/HalfOpen with configurable threshold/window/cooldown
//! - Bulkhead: bounded concurrency via Semaphore
//! - Retry: jittered exponential backoff with attempt budget
//! - Timeout: per-request timeout wrapper
//! - Idempotency: deduplication by (subject, key) with body-hash conflict detection
//! - Degradation mode: Normal / Degraded / ReadOnlyLockdown shared flag

pub mod error;
pub mod degradation;
pub mod circuit_breaker;
pub mod bulkhead;
pub mod retry;
pub mod timeout;
pub mod idempotency;

pub use error::ResilienceError;
pub use degradation::{DegradationFlag, DegradationMode};
pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use bulkhead::Bulkhead;
pub use retry::{RetryConfig, JitterKind, with_retry};
pub use timeout::with_timeout;
pub use idempotency::IdempotencyStore;
