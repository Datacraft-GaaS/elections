// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Resilience primitives for the eel platform (ADR-0009).
//!
//! - Circuit breaker: Closed/Open/HalfOpen with configurable threshold/window/cooldown
//! - Bulkhead: bounded concurrency via Semaphore
//! - Retry: jittered exponential backoff with attempt budget
//! - Timeout: per-request timeout wrapper
//! - Idempotency: deduplication by (subject, key) with body-hash conflict detection
//! - Degradation mode: Normal / Degraded / ReadOnlyLockdown shared flag
#![warn(missing_docs)]

pub mod bulkhead;
pub mod circuit_breaker;
pub mod degradation;
pub mod error;
pub mod idempotency;
pub mod retry;
pub mod timeout;

pub use bulkhead::Bulkhead;
pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use degradation::{DegradationFlag, DegradationMode};
pub use error::ResilienceError;
pub use idempotency::IdempotencyStore;
pub use retry::{with_retry, JitterKind, RetryConfig};
pub use timeout::with_timeout;
