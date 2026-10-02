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

//! Error types for search planner.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlannerError {
    #[error("no shards available")]
    NoShards,

    #[error("no executable steps for query")]
    NoSteps,

    #[error("early termination threshold {0} not in [0, 1)")]
    InvalidThreshold(f64),

    #[error("execution error on shard {shard}: {reason}")]
    ShardExecution { shard: uuid::Uuid, reason: String },
}
