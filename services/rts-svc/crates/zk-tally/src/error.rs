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

//! Error types for the zk-tally crate.

/// Errors produced by zk-tally operations.
#[derive(Debug, thiserror::Error)]
pub enum ZkError {
    #[error("constraint synthesis failed: {0}")]
    ConstraintSynthesis(String),

    #[error("trusted setup failed: {0}")]
    Setup(String),

    #[error("proof generation failed: {0}")]
    Prove(String),

    #[error("proof verification failed: {0}")]
    Verify(String),

    #[error("serialisation error: {0}")]
    Serialise(String),

    #[error("leaf count {0} exceeds maximum {1}")]
    TooManyLeaves(usize, usize),

    #[error("tally mismatch: leaves sum to {leaves_sum}, published = {published}")]
    TallyMismatch { leaves_sum: u64, published: u64 },
}
