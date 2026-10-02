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

//! Per-thread ONNX session pool.
//!
//! Maintains up to `max_sessions` embedder instances.  Each thread checks out
//! one instance (or creates a new one if below the limit) and returns it when
//! the guard is dropped.

use crate::{embed::DeepPrintEmbedder, DlError};
use std::sync::{Arc, Mutex};

/// A pool of `DeepPrintEmbedder` instances.
pub struct EmbedderPool {
    /// Available (idle) embedders.
    idle: Arc<Mutex<Vec<DeepPrintEmbedder>>>,
    /// Maximum number of sessions (idle + checked-out).
    max_sessions: usize,
    /// Current total count (idle + checked-out).
    total: Arc<Mutex<usize>>,
    model_path: Option<std::path::PathBuf>,
}

impl EmbedderPool {
    pub fn new(max_sessions: usize, model_path: Option<std::path::PathBuf>) -> Self {
        Self {
            idle: Arc::new(Mutex::new(Vec::new())),
            max_sessions,
            total: Arc::new(Mutex::new(0)),
            model_path,
        }
    }

    /// Check out an embedder.  Creates a new one if the pool is not full.
    /// Returns an error if the pool is exhausted.
    pub fn checkout(&self) -> Result<PooledEmbedder, DlError> {
        let mut idle = self.idle.lock().expect("mutex not poisoned");
        if let Some(e) = idle.pop() {
            return Ok(PooledEmbedder {
                embedder: Some(e),
                pool: Arc::clone(&self.idle),
            });
        }
        let mut total = self.total.lock().expect("mutex not poisoned");
        if *total >= self.max_sessions {
            return Err(DlError::PoolExhausted);
        }
        *total += 1;
        let e = match &self.model_path {
            Some(p) => DeepPrintEmbedder::from_path(p.clone()),
            None => DeepPrintEmbedder::from_env(),
        };
        Ok(PooledEmbedder {
            embedder: Some(e),
            pool: Arc::clone(&self.idle),
        })
    }
}

/// RAII guard: returns the embedder to the pool on drop.
pub struct PooledEmbedder {
    embedder: Option<DeepPrintEmbedder>,
    pool: Arc<Mutex<Vec<DeepPrintEmbedder>>>,
}

impl PooledEmbedder {
    pub fn get(&self) -> &DeepPrintEmbedder {
        self.embedder
            .as_ref()
            .expect("embedder missing from pool guard")
    }
}

impl Drop for PooledEmbedder {
    fn drop(&mut self) {
        if let Some(e) = self.embedder.take() {
            self.pool.lock().expect("mutex not poisoned").push(e);
        }
    }
}
