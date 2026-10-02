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

//! Token-bucket rate limiter for sequencer back-pressure.

use std::sync::Arc;
use tokio::sync::Semaphore;

/// Bounds the number of concurrent append operations.
/// Producers that cannot acquire a permit receive 429 + Retry-After.
pub struct SequencerRateLimiter {
    sem: Arc<Semaphore>,
    max_per_sec: u32,
}

impl SequencerRateLimiter {
    pub fn new(max_per_sec: u32) -> Self {
        Self {
            sem: Arc::new(Semaphore::new(max_per_sec as usize)),
            max_per_sec,
        }
    }

    /// Try to acquire a permit. Returns None if rate limit exceeded.
    pub fn try_acquire(&self) -> Option<tokio::sync::SemaphorePermit<'_>> {
        self.sem.try_acquire().ok()
    }

    pub fn max_per_sec(&self) -> u32 {
        self.max_per_sec
    }
}
