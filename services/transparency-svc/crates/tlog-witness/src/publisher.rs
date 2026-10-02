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

//! WitnessPublisher trait and NoopPublisher.
//!
//! WP-4.12 wires the Commission website, GitHub, IPFS-KE, Gazette, and KBC channels.
//! Here we define the contract only.

use async_trait::async_trait;
use tlog_domain::SignedTreeHead;

/// Publish an STH to an external channel.
#[async_trait]
pub trait WitnessPublisher: Send + Sync {
    /// Publish the STH to the channel. Must be idempotent.
    async fn publish(&self, sth: &SignedTreeHead) -> Result<(), PublisherError>;
    fn channel_name(&self) -> &str;
}

/// No-op publisher for staging / unit tests.
pub struct NoopPublisher;

#[async_trait]
impl WitnessPublisher for NoopPublisher {
    async fn publish(&self, _sth: &SignedTreeHead) -> Result<(), PublisherError> {
        Ok(())
    }
    fn channel_name(&self) -> &str {
        "noop"
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PublisherError {
    #[error("network error: {0}")]
    Network(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
