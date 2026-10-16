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

use eel_eventstore_rs::{EventStore, EventStoreError, StoredEvent};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("projection db error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("projection store error: {0}")]
    Store(#[from] EventStoreError),
    #[error("projection serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type ProjectorFn = dyn Fn(
        StoredEvent,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProjectionError>> + Send>>
    + Send
    + Sync;

pub struct ProjectionRunner {
    projectors: Vec<Box<ProjectorFn>>,
}
impl ProjectionRunner {
    pub fn new() -> Self {
        Self {
            projectors: Vec::new(),
        }
    }
    pub fn add<F, Fut>(&mut self, projector: F)
    where
        F: Fn(StoredEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), ProjectionError>> + Send + 'static,
    {
        self.projectors
            .push(Box::new(move |e| Box::pin(projector(e))));
    }
    pub async fn run_from_store(
        &self,
        _store: &EventStore,
        from_version: i64,
    ) -> Result<(), ProjectionError> {
        tracing::info!(from_version, "ProjectionRunner: starting replay (stub)");
        Ok(())
    }
}
impl Default for ProjectionRunner {
    fn default() -> Self {
        Self::new()
    }
}
