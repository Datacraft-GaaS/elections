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

/// Errors produced by CBEFF container parsing and serialisation.
#[derive(Debug, Error)]
pub enum CbeffError {
    #[error("invalid magic bytes (expected CBEFF header)")]
    InvalidMagic,

    #[error("invalid or truncated length field")]
    InvalidLength,

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("nested CBEFF error: {0}")]
    NestedError(#[from] Box<CbeffError>),
}
