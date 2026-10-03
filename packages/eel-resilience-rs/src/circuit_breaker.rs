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

use crate::error::ResilienceError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
struct Inner {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    window_start: Instant,
    window_failures: u32,
}

/// Circuit breaker for a single dependency.
///
/// - Closed: requests pass through; tracks failures in a time window.
/// - Open: requests fail fast; after cooldown transitions to HalfOpen.
/// - HalfOpen: one probe request let through; success -> Closed, failure -> Open.
#[derive(Clone)]
pub struct CircuitBreaker {
    dependency: String,
    /// Number of failures in `window` before opening.
    threshold: u32,
    /// Time window for counting failures.
    window: Duration,
    /// How long to wait in Open before probing.
    cooldown: Duration,
    inner: Arc<Mutex<Inner>>,
}

impl CircuitBreaker {
    pub fn new(
        dependency: impl Into<String>,
        threshold: u32,
        window: Duration,
        cooldown: Duration,
    ) -> Self {
        Self {
            dependency: dependency.into(),
            threshold,
            window,
            cooldown,
            inner: Arc::new(Mutex::new(Inner {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure: None,
                window_start: Instant::now(),
                window_failures: 0,
            })),
        }
    }

    /// Returns Err if circuit is open.
    pub async fn allow(&self) -> Result<(), ResilienceError> {
        let mut inner = self.inner.lock().await;
        match inner.state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if cooldown elapsed -> HalfOpen
                if let Some(last) = inner.last_failure {
                    if last.elapsed() >= self.cooldown {
                        inner.state = CircuitState::HalfOpen;
                        return Ok(());
                    }
                }
                Err(ResilienceError::CircuitOpen {
                    dependency: self.dependency.clone(),
                })
            }
            CircuitState::HalfOpen => Ok(()), // allow probe
        }
    }

    pub async fn record_success(&self) {
        let mut inner = self.inner.lock().await;
        inner.success_count += 1;
        if inner.state == CircuitState::HalfOpen {
            inner.state = CircuitState::Closed;
            inner.window_failures = 0;
            inner.window_start = Instant::now();
        }
    }

    pub async fn record_failure(&self) {
        let mut inner = self.inner.lock().await;
        inner.failure_count += 1;
        inner.last_failure = Some(Instant::now());

        // Reset window if elapsed
        if inner.window_start.elapsed() > self.window {
            inner.window_failures = 0;
            inner.window_start = Instant::now();
        }
        inner.window_failures += 1;

        if inner.window_failures >= self.threshold || inner.state == CircuitState::HalfOpen {
            inner.state = CircuitState::Open;
        }
    }

    pub async fn state(&self) -> CircuitState {
        self.inner.lock().await.state
    }
}
