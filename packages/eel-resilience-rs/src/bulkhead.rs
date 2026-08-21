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
