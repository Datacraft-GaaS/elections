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

//! Error types for the enrolment crate.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnrolmentError {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),

    #[error("event build failed: {0}")]
    EventBuild(String),

    #[error("quality check failed: {0}")]
    QualityCheckFailed(String),

    #[error("dedup search failed: {0}")]
    DedupFailed(String),

    #[error("dedup result is missing; call dedup_search before route_decision")]
    MissingDedupResult,

    #[error("job not found: {0}")]
    JobNotFound(uuid::Uuid),

    #[error("duplicate enrolment job: subject={0} source={1} captured_at={2}")]
    DuplicateJob(String, String, String),
}
