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

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReproError {
    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("archive corrupt: {0}")]
    ArchiveCorrupt(String),

    #[error("version mismatch: bundle format {bundle_ver}, parser {parser_ver}")]
    VersionMismatch {
        bundle_ver: String,
        parser_ver: String,
    },

    #[error("assembly already exists for election {election_id} version {version}")]
    AlreadyAssembled {
        election_id: String,
        version: String,
    },

    #[error("component missing: {0}")]
    ComponentMissing(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}
