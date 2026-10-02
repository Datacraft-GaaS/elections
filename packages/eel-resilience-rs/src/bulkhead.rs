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

use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use crate::error::ResilienceError;

/// Bounded concurrency wrapper for an upstream dependency.
#[derive(Clone)]
pub struct Bulkhead {
    dependency: String,
    semaphore: Arc<Semaphore>,
}

impl Bulkhead {
    pub fn new(dependency: impl Into<String>, max_concurrent: usize) -> Self {
        Self {
            dependency: dependency.into(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Acquire a permit, failing immediately if at capacity.
    pub async fn acquire(&self) -> Result<BulkheadPermit, ResilienceError> {
        self.semaphore.clone()
            .try_acquire_owned()
            .map(|permit| BulkheadPermit { _permit: permit })
            .map_err(|_| ResilienceError::BulkheadFull { dependency: self.dependency.clone() })
    }

    /// Available slots.
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// RAII handle that releases the permit on drop.
pub struct BulkheadPermit {
    _permit: OwnedSemaphorePermit,
}
